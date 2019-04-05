//  Main program for the sensor app that reads sensor data and sends to the cloud
/**
 * Depending on the type of package, there are different
 * compilation rules for this directory.  This comment applies
 * to packages of type "app."  For other types of packages,
 * please view the documentation at http://mynewt.apache.org/.
 *
 * Put source files in this directory.  All files that have a *.c
 * ending are recursively compiled in the src/ directory and its
 * descendants.  The exception here is the arch/ directory, which
 * is ignored in the default compilation.
 *
 * The arch/<your-arch>/ directories are manually added and
 * recursively compiled for all files that end with either *.c
 * or *.a.  Any directories in arch/ that don't match the
 * architecture being compiled are not compiled.
 *
 * Architecture is set by the BSP/MCU combination.
 */
#include <sysinit/sysinit.h>
#include <os/os.h>
#include <defs/error.h>
#include <sensor/sensor.h>
#include <sensor/temperature.h>
//#include <oic/oc_api.h>
//#include <oic/port/mynewt/ip.h>
#include <console/console.h>  //  Actually points to libs/semihosting_console
#include "sensor_coap.h"
#include "esp8266_driver.h"

//  CoAP Connection Settings e.g. coap://coap.thethings.io/v2/things/IVRiBCcR6HPp_CcZIFfOZFxz_izni5xc_KO-kgSA2Y8
#define COAP_HOST   "coap.thethings.io"  //  CoAP hostname e.g. coap.thethings.io
#define COAP_PORT   COAP_PORT_UNSECURED  //  CoAP port, usually UDP port 5683
#define COAP_URI    "v2/things/IVRiBCcR6HPp_CcZIFfOZFxz_izni5xc_KO-kgSA2Y8"  //  CoAP URI

#define TRANSCEIVER_DEVICE ESP8266_DEVICE  //  Name of the transceiver device e.g. esp8266_0
#define MAX_WIFI_AP 3  //  Read at most 3 WiFi access points.

//  static void init_sensors(void);
static int init_tasks(void);

static nsapi_wifi_ap_t wifi_aps[MAX_WIFI_AP];  //  List of scanned WiFi access points.

int __wrap_coap_receive(/* struct os_mbuf **mp */) {
    //  We override the default coap_receive() with an empty function so that we will 
    //  NOT pull in any modules for receiving and parse CoAP requests.  We only need
    //  to transmit CoAP requests.
    console_printf("__wrap_coap_receive\n"); console_flush();
    return -1;
}

//  CoAP Connection Configuration
static struct esp8266_server_handle coap_server = {
    .endpoint = {
        .host = COAP_HOST,  //  CoAP hostname e.g. coap.thethings.io
        .port = COAP_PORT   //  CoAP port, usually UDP port 5683
    }
};

static void send_coap_request(void) {
    //  Send the sensor data over CoAP to the cloud.
    esp8266_register_transport();                  //  Register the ESP8266 driver as a transport for CoAP.
    init_esp8266_endpoint(&coap_server.endpoint);  //  Init the endpoint before use.

    //  Create a CoAP request.
    int rc = init_sensor_post((oc_server_handle_t *) &coap_server, COAP_URI);
    assert(rc != 0);

    //  Populate the CoAP request body in JSON format.
    //  For thethings.io, the body should look like {"values":[{"key":"tmp","value":28.7}, ... ]}
    rep_start_root_object();                              //  Create the root.
        rep_set_array(root, values);                      //  Create "values" as an array of objects.
            rep_object_array_start_item(values);          //  Create a new item in the "values" array.
                //  Each child of "values" is an object like {"key":"tmp","value":28.7}.
                rep_set_text_string(values, key,   "tmp");  //  Set the key.
                rep_set_double     (values, value, 28.2);   //  Set the value.
            rep_object_array_end_item(values);            //  Close the item in the "values" array.
        rep_close_array(root, values);                    //  Close the "values" array.
    rep_end_root_object();                                //  Close the root.

    //  Forward the CoAP request to the CoAP Background Task for transmission.
    rc = do_sensor_post();  assert(rc);
    console_printf("Sending POST request\n");
}

int main(int argc, char **argv) {
    int rc;
    sysinit();           //  Initialize all packages.  Create the sensors.
    //  init_sensors();  //  Init the sensors.    
    init_esp8266();      //  Init the ESP8266 transceiver.
    send_coap_request(); //  Send CoAP request.
    rc = init_tasks();  assert(rc == 0);  //  Start the background tasks.   
    while (1) {                   //  Loop forever...
        os_eventq_run(            //  Process events...
            os_eventq_dflt_get()  //  From default event queue.
        );
    }
    return 0;  //  Never comes here.
}

#define WORK_TASK_PRIO (10)  //  Command task is lower priority than event processing.  
#define WORK_STACK_SIZE OS_STACK_ALIGN(256)

static struct os_task work_task;
static uint8_t work_stack[sizeof(os_stack_t) * WORK_STACK_SIZE];

static void work_task_handler(void *arg) {
    struct sensor *trans = sensor_mgr_find_next_bydevname(TRANSCEIVER_DEVICE, NULL);
    assert(trans != NULL);

    int rc = esp8266_scan(&trans->s_itf, wifi_aps, MAX_WIFI_AP);
    assert(rc > 0 && rc <= MAX_WIFI_AP);

    while (1) {  //  Loop forever...        
        os_time_delay(1000);  //  Wait one second.
    }
}

static int init_tasks(void) {
    os_task_init(&work_task, "work", work_task_handler, NULL,
            WORK_TASK_PRIO, OS_WAIT_FOREVER, 
            (os_stack_t *) work_stack, WORK_STACK_SIZE);
    return 0;
}

#ifdef POLL_SENSOR
    #define MY_SENSOR_DEVICE "bme280_0"
    #define MY_SENSOR_POLL_TIME 2000
    #define LISTENER_CB 1
    #define READ_CB 2

    static int read_temperature(struct sensor* sensor, void *arg, void *databuf, sensor_type_t type);

    static struct sensor *my_sensor;

    static struct sensor_listener listener = {
        .sl_sensor_type = SENSOR_TYPE_AMBIENT_TEMPERATURE,
        .sl_func = read_temperature,
        .sl_arg = (void *)LISTENER_CB,
    };

    static void init_sensors(void) {
        int rc;
        rc = sensor_set_poll_rate_ms(MY_SENSOR_DEVICE, MY_SENSOR_POLL_TIME);
        assert(rc == 0);

        my_sensor = sensor_mgr_find_next_bydevname(MY_SENSOR_DEVICE, NULL);
        assert(my_sensor != NULL);

        rc = sensor_register_listener(my_sensor, &listener);
        assert(rc == 0);
    }

    static int read_temperature(struct sensor* sensor, void *arg, void *databuf, sensor_type_t type) {
        struct sensor_temp_data *temp;
        if (!databuf) { return SYS_EINVAL; }
        temp = (struct sensor_temp_data *)databuf;
        if (!temp->std_temp_is_valid) { return SYS_EINVAL; }
        console_printf(
            "temp = %d.%d\n",
            (int) (temp->std_temp),
            (int) (10.0 * temp->std_temp) % 10
        );
        return 0;
    }
#endif  //  POLL_SENSOR

//  Dummy destructor for global C++ objects, since our program never terminates.  From https://arobenko.gitbooks.io/bare_metal_cpp/content/compiler_output/static.html.
void* __dso_handle = NULL;
void _fini(void) { }
int __aeabi_atexit(void *object, void (*destructor)(void *), void *dso_handle) { return 0; }
