    //!  Send sensor data to a CoAP Server or a Collector Node.  The CoAP payload will be encoded as JSON
    //!  for CoAP Server and CBOR for Collector Node.  The sensor data will be transmitted to 
    //!  CoAP Server over WiFi via the ESP8266 transceiver, and to Collector Node via nRF24L01 transceiver.
    //!  This enables transmission of Sensor Data to a local Sensor Network (via nRF24L01)
    //!  and to the internet (via ESP8266).  For sending to Collector Node we use raw temperature (integer) 
    //!  instead of computed temperature (floating-point) to make the encoding simpler and faster.
    //!  Note that we are using a patched version of apps/my_sensor_app/src/vsscanf.c that
    //!  fixes ESP8266 response parsing bugs.  The patched file must be present in that location.
    //!  This is the Rust version of `https://github.com/lupyuen/stm32bluepill-mynewt-sensor/blob/rust/apps/my_sensor_app/OLDsrc/send_coap.c`
    use cstr_core::CStr;
    use cty::*;
    use crate::{coap, d, fill_zero};
    use crate::base::*;
    use crate::mynewt::{result::*, kernel::os::{self, os_task, os_stack_t},
                        encoding::{json_context::{self, JSON_CONTEXT,
                                                  ToBytesOptionalNull},
                                   tinycbor},
                        libs::{mynewt_rust, sensor_network,
                               sensor_coap::{self, sensor_value}}};
    fn test_safe_wrap() {
        "-------------------------------------------------------------";
        "-------------------------------------------------------------";
    }
    /// Run a block of CBOR encoding calls with error checking.
    fn test_run() {
        "-------------------------------------------------------------";
        let encoder = JSON_CONTEXT.encoder("JSON_CONTEXT", "_map");
        let res =
            tinycbor::cbor_encode_text_string(encoder,
                                              JSON_CONTEXT.key_to_cstr(key_with_opt_null),
                                              JSON_CONTEXT.cstr_len(key_with_opt_null));
        JSON_CONTEXT.check_error(res);
        let res = tinycbor::cbor_encode_int(encoder, value);
        JSON_CONTEXT.check_error(res);
        "-------------------------------------------------------------";
    }
    ///  Storage for Network Task: Mynewt task object will be saved here.
    static mut NETWORK_TASK: os_task =
        unsafe {
            ::core::mem::transmute::<[u8; ::core::mem::size_of::<os_task>()],
                                     os_task>([0;
                                                  ::core::mem::size_of::<os_task>()])
        };
    ///  Stack space for Network Task, initialised to 0.
    static mut NETWORK_TASK_STACK: [os_stack_t; NETWORK_TASK_STACK_SIZE] =
        [0; NETWORK_TASK_STACK_SIZE];
    ///  Size of the stack (in 4-byte units). Previously `OS_STACK_ALIGN(256)`  
    const NETWORK_TASK_STACK_SIZE: usize = 256;
    ///  Set to true when network tasks have been completed
    static mut NETWORK_IS_READY: bool = false;
    ///  Start the Network Task in the background.  The Network Task prepares the network drivers
    ///  (ESP8266 and nRF24L01) for transmitting sensor data messages.  
    ///  Connecting the ESP8266 to the WiFi access point may be slow so we do this in the background.
    ///  Also perform WiFi Geolocation if it is enabled.  Return 0 if successful.
    pub fn start_network_task() -> MynewtResult<()> {
        console_print(b"NET start\n");
        let rc =
            unsafe {
                os::os_task_init(&mut NETWORK_TASK,
                                 b"network\0".as_ptr() as *const c_char,
                                 Some(network_task_func),
                                 0 as *mut ::cty::c_void, 10,
                                 os::OS_WAIT_FOREVER as u32,
                                 NETWORK_TASK_STACK.as_ptr() as
                                     *mut os_stack_t,
                                 NETWORK_TASK_STACK_SIZE as u16)
            };
        {
            match (&rc, &0) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        {
                            ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                          "`,\n right: `",
                                                                                          "`"],
                                                                                        &match (&&*left_val,
                                                                                                &&*right_val)
                                                                                             {
                                                                                             (arg0,
                                                                                              arg1)
                                                                                             =>
                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                           ::core::fmt::Debug::fmt),
                                                                                              ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                           ::core::fmt::Debug::fmt)],
                                                                                         }),
                                                         &("src/send_coap.rs",
                                                           129u32, 5u32))
                        }
                    }
                }
            }
        };
        Ok(())
    }
    ///  Network Task runs this function in the background to prepare the network drivers
    ///  (ESP8266 and nRF24L01) for transmitting sensor data messages.  Also perform WiFi Geolocation if it is enabled.
    ///  For Collector Node and Standalone Node: We connect the ESP8266 to the WiFi access point. 
    ///  Connecting the ESP8266 to the WiFi access point may be slow so we do this in the background.
    ///  Register the ESP8266 driver as the network transport for CoAP Server.  
    ///  For Collector Node and Sensor Nodes: We register the nRF24L01 driver as the network transport for 
    ///  CoAP Collector.
    extern "C" fn network_task_func(_arg: *mut ::cty::c_void) {
        console_print(b"NET start\n");
        if !unsafe { !NETWORK_IS_READY } {
            {
                ::core::panicking::panic(&("assertion failed: unsafe { !NETWORK_IS_READY }",
                                           "src/send_coap.rs", 141u32, 37u32))
            }
        };
        if unsafe {
               sensor_network::is_standalone_node() ||
                   sensor_network::is_collector_node()
           } {
            let rc = unsafe { sensor_network::register_server_transport() };
            {
                match (&rc, &0) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                              "`,\n right: `",
                                                                                              "`"],
                                                                                            &match (&&*left_val,
                                                                                                    &&*right_val)
                                                                                                 {
                                                                                                 (arg0,
                                                                                                  arg1)
                                                                                                 =>
                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                               ::core::fmt::Debug::fmt),
                                                                                                  ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                               ::core::fmt::Debug::fmt)],
                                                                                             }),
                                                             &("src/send_coap.rs",
                                                               149u32, 75u32))
                            }
                        }
                    }
                }
            };
        }
        if unsafe {
               sensor_network::is_collector_node() ||
                   sensor_network::is_sensor_node()
           } {
            let rc =
                unsafe { sensor_network::register_collector_transport() };
            {
                match (&rc, &0) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                              "`,\n right: `",
                                                                                              "`"],
                                                                                            &match (&&*left_val,
                                                                                                    &&*right_val)
                                                                                                 {
                                                                                                 (arg0,
                                                                                                  arg1)
                                                                                                 =>
                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                               ::core::fmt::Debug::fmt),
                                                                                                  ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                               ::core::fmt::Debug::fmt)],
                                                                                             }),
                                                             &("src/send_coap.rs",
                                                               157u32, 78u32))
                            }
                        }
                    }
                }
            };
        }
        unsafe { NETWORK_IS_READY = true; }
        loop  {
            console_print(b"NET free mbuf %d\n");
            unsafe { os::os_time_delay(10 * os::OS_TICKS_PER_SEC); }
        }
    }
    ///  Compose a CoAP message (CBOR or JSON) with the sensor value in `val` and transmit to the
    ///  Collector Node (if this is a Sensor Node) or to the CoAP Server (if this is a Collector Node
    ///  or Standalone Node).  
    ///  For Sensor Node or Standalone Node: sensor_node is the sensor name (`bme280_0` or `temp_stm32_0`)
    ///  For Collector Node: sensor_node is the Sensor Node Address of the Sensor Node that transmitted
    ///  the sensor data (like `b3b4b5b6f1`)
    ///  The message will be enqueued for transmission by the CoAP / OIC Background Task 
    ///  so this function will return without waiting for the message to be transmitted.  
    ///  Return 0 if successful, SYS_EAGAIN if network is not ready yet.
    pub fn send_sensor_data(sensor_val: &SensorValue, sensor_node: &CStr)
     -> MynewtResult<()> {
        console_print(b"send_sensor_data\n");
        let mut val =
            unsafe {
                ::core::mem::transmute::<[u8; ::core::mem::size_of::<sensor_value>()],
                                         sensor_value>([0;
                                                           ::core::mem::size_of::<sensor_value>()])
            };
        if unsafe {
               sensor_network::should_send_to_collector(&mut val,
                                                        sensor_node.as_ptr())
           } {
            return send_sensor_data_to_collector(sensor_val, sensor_node);
        }
        send_sensor_data_to_server(sensor_val, sensor_node)
    }
    ///  Compose a CoAP JSON message with the Sensor Key (field name) and Value in val 
    ///  and send to the CoAP server and URI.  The Sensor Value may be integer or float.
    ///  For temperature, the Sensor Key is either `t` for raw temperature (integer, from 0 to 4095) 
    ///  or `tmp` for computed temperature (float).
    ///  The message will be enqueued for transmission by the CoAP / OIC 
    ///  Background Task so this function will return without waiting for the message 
    ///  to be transmitted.  Return 0 if successful, `SYS_EAGAIN` if network is not ready yet.
    ///  For the CoAP server hosted at thethings.io, the CoAP payload should be encoded in JSON like this:
    ///  ```
    ///  {"values":[
    ///    {"key":"device", "value":"0102030405060708090a0b0c0d0e0f10"},
    ///    {"key":"tmp",    "value":28.7},
    ///    {"key":"...",    "value":... },
    ///    ... ]}
    ///  ```
    fn send_sensor_data_to_server(sensor_val: &SensorValue, node_id: &CStr)
     -> MynewtResult<()> {
        if let SensorValueType::None = sensor_val.val {
            if !false {
                {
                    ::core::panicking::panic(&("assertion failed: false",
                                               "src/send_coap.rs", 211u32,
                                               53u32))
                }
            };
        }
        {
            match (&node_id.to_bytes()[0], &0) {
                (left_val, right_val) => {
                    if *left_val == *right_val {
                        {
                            ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(&["assertion failed: `(left != right)`\n  left: `",
                                                                                          "`,\n right: `",
                                                                                          "`"],
                                                                                        &match (&&*left_val,
                                                                                                &&*right_val)
                                                                                             {
                                                                                             (arg0,
                                                                                              arg1)
                                                                                             =>
                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                           ::core::fmt::Debug::fmt),
                                                                                              ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                           ::core::fmt::Debug::fmt)],
                                                                                         }),
                                                         &("src/send_coap.rs",
                                                           213u32, 5u32))
                        }
                    }
                }
            }
        };
        if unsafe { !NETWORK_IS_READY } {
            return Err(MynewtError::SYS_EAGAIN);
        }
        let device_id_ptr = unsafe { sensor_network::get_device_id() };
        let device_id: &CStr = unsafe { CStr::from_ptr(device_id_ptr) };
        let rc =
            unsafe { sensor_network::init_server_post(0 as *const c_char) };
        if !rc {
            {
                ::core::panicking::panic(&("assertion failed: rc",
                                           "src/send_coap.rs", 221u32, 80u32))
            }
        };
        let _payload =
            {
                "begin json root";
                {
                    "begin json coap_root";
                    unsafe { sensor_coap::json_rep_start_root_object() }
                    {
                        {
                            "begin json coap_array , object : JSON_CONTEXT , key : values";
                            {
                                "<< jarri , o: JSON_CONTEXT, k: values";
                                let key_with_null: &str = "values\u{0}";
                                unsafe {
                                    mynewt_rust::json_helper_set_array(JSON_CONTEXT.to_void_ptr(),
                                                                       JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()));
                                };
                            };
                            {
                                " >>  >> \"device\" >> : device_id , \"node\" : node_id , sensor_val ,";
                                "add1 key : \"device\" value : $crate::parse!(@ json device_id) to object :\nJSON_CONTEXT";
                                {
                                    "begin json coap_item_str , parent : JSON_CONTEXT , key : \"device\" , val :\n$crate::parse!(@ json device_id)";
                                    {
                                        "begin json coap_item , array : JSON_CONTEXT";
                                        {
                                            "<< jitmi c: JSON_CONTEXT";
                                            let key_with_null: &str =
                                                "JSON_CONTEXT\u{0}";
                                            unsafe {
                                                mynewt_rust::json_helper_object_array_start_item(JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                            };
                                        };
                                        {
                                            {
                                                "-- jtxti o: JSON_CONTEXT, k: key, v: \"device\"";
                                                let key_with_null: &str =
                                                    "key\u{0}";
                                                let value_with_opt_null:
                                                        &[u8] =
                                                    "device".to_bytes_optional_nul();
                                                unsafe {
                                                    mynewt_rust::json_helper_set_text_string(JSON_CONTEXT.to_void_ptr(),
                                                                                             JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()),
                                                                                             JSON_CONTEXT.value_to_cstr(value_with_opt_null))
                                                };
                                            };
                                            {
                                                "-- jtxti o: JSON_CONTEXT, k: value, v: $crate::parse!(@ json device_id)";
                                                let key_with_null: &str =
                                                    "value\u{0}";
                                                let value_with_opt_null:
                                                        &[u8] =
                                                    device_id.to_bytes_optional_nul();
                                                unsafe {
                                                    mynewt_rust::json_helper_set_text_string(JSON_CONTEXT.to_void_ptr(),
                                                                                             JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()),
                                                                                             JSON_CONTEXT.value_to_cstr(value_with_opt_null))
                                                };
                                            };
                                        };
                                        {
                                            ">>";
                                            let key_with_null: &str =
                                                "JSON_CONTEXT\u{0}";
                                            unsafe {
                                                mynewt_rust::json_helper_object_array_end_item(JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                            };
                                        };
                                        "end json coap_item";
                                    };
                                    "end json coap_item_str";
                                };
                                "--------------------";
                                " >>  >> \"node\" >> : node_id , sensor_val ,";
                                "add1 key : \"node\" value : $crate::parse!(@ json node_id) to object :\nJSON_CONTEXT";
                                {
                                    "begin json coap_item_str , parent : JSON_CONTEXT , key : \"node\" , val :\n$crate::parse!(@ json node_id)";
                                    {
                                        "begin json coap_item , array : JSON_CONTEXT";
                                        {
                                            "<< jitmi c: JSON_CONTEXT";
                                            let key_with_null: &str =
                                                "JSON_CONTEXT\u{0}";
                                            unsafe {
                                                mynewt_rust::json_helper_object_array_start_item(JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                            };
                                        };
                                        {
                                            {
                                                "-- jtxti o: JSON_CONTEXT, k: key, v: \"node\"";
                                                let key_with_null: &str =
                                                    "key\u{0}";
                                                let value_with_opt_null:
                                                        &[u8] =
                                                    "node".to_bytes_optional_nul();
                                                unsafe {
                                                    mynewt_rust::json_helper_set_text_string(JSON_CONTEXT.to_void_ptr(),
                                                                                             JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()),
                                                                                             JSON_CONTEXT.value_to_cstr(value_with_opt_null))
                                                };
                                            };
                                            {
                                                "-- jtxti o: JSON_CONTEXT, k: value, v: $crate::parse!(@ json node_id)";
                                                let key_with_null: &str =
                                                    "value\u{0}";
                                                let value_with_opt_null:
                                                        &[u8] =
                                                    node_id.to_bytes_optional_nul();
                                                unsafe {
                                                    mynewt_rust::json_helper_set_text_string(JSON_CONTEXT.to_void_ptr(),
                                                                                             JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()),
                                                                                             JSON_CONTEXT.value_to_cstr(value_with_opt_null))
                                                };
                                            };
                                        };
                                        {
                                            ">>";
                                            let key_with_null: &str =
                                                "JSON_CONTEXT\u{0}";
                                            unsafe {
                                                mynewt_rust::json_helper_object_array_end_item(JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                            };
                                        };
                                        "end json coap_item";
                                    };
                                    "end json coap_item_str";
                                };
                                "--------------------";
                                " >>  >> sensor_val >> ,";
                                "--------------------";
                                {
                                    "begin json coap_item_int_val , c : JSON_CONTEXT , val : sensor_val";
                                    if let SensorValueType::Uint(val) =
                                           sensor_val.val {
                                        {
                                            "begin json coap_item_int , key : sensor_val.key , value : val";
                                            {
                                                "begin json coap_item , array : JSON_CONTEXT";
                                                {
                                                    "<< jitmi c: JSON_CONTEXT";
                                                    let key_with_null: &str =
                                                        "JSON_CONTEXT\u{0}";
                                                    unsafe {
                                                        mynewt_rust::json_helper_object_array_start_item(JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                                    };
                                                };
                                                {
                                                    {
                                                        "-- jtxte o: JSON_CONTEXT, k: \"key\", v: sensor_val.key";
                                                        let key_with_opt_null:
                                                                &[u8] =
                                                            "key".to_bytes_optional_nul();
                                                        let value_with_opt_null:
                                                                &[u8] =
                                                            sensor_val.key.to_bytes_optional_nul();
                                                        unsafe {
                                                            mynewt_rust::json_helper_set_text_string(JSON_CONTEXT.to_void_ptr(),
                                                                                                     JSON_CONTEXT.key_to_cstr(key_with_opt_null),
                                                                                                     JSON_CONTEXT.value_to_cstr(value_with_opt_null))
                                                        };
                                                    };
                                                    {
                                                        "-- jinte o: JSON_CONTEXT, k: \"value\", v: val";
                                                        let key_with_opt_null:
                                                                &[u8] =
                                                            "value".to_bytes_optional_nul();
                                                        let value =
                                                            val as u64;
                                                        unsafe {
                                                            mynewt_rust::json_helper_set_int(JSON_CONTEXT.to_void_ptr(),
                                                                                             JSON_CONTEXT.key_to_cstr(key_with_opt_null),
                                                                                             value)
                                                        };
                                                    };
                                                };
                                                {
                                                    ">>";
                                                    let key_with_null: &str =
                                                        "JSON_CONTEXT\u{0}";
                                                    unsafe {
                                                        mynewt_rust::json_helper_object_array_end_item(JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                                    };
                                                };
                                                "end json coap_item";
                                            };
                                            "end json coap_item_int";
                                        };
                                    } else {
                                        unsafe {
                                            JSON_CONTEXT.fail(json_context::JsonError::VALUE_NOT_UINT)
                                        };
                                    }
                                    "end json coap_item_int_val";
                                };
                                "--------------------";
                            };
                            {
                                ">>";
                                let key_with_null: &str = "values\u{0}";
                                unsafe {
                                    mynewt_rust::json_helper_close_array(JSON_CONTEXT.to_void_ptr(),
                                                                         JSON_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                };
                            };
                            "end json coap_array";
                        };
                    };
                    unsafe { sensor_coap::json_rep_end_root_object() }
                    "end json coap_root";
                };
                "end json root";
                ()
            };
        let rc = unsafe { sensor_network::do_server_post() };
        if !rc {
            {
                ::core::panicking::panic(&("assertion failed: rc",
                                           "src/send_coap.rs", 240u32, 60u32))
            }
        };
        console_print(b"NET view your sensor at \nhttps://blue-pill-geolocate.appspot.com?device=%s\n");
        Ok(())
    }
    ///  Compose a CoAP CBOR message with the Sensor Key (field name) and Value in val and 
    ///  transmit to the Collector Node.  The Sensor Value should be integer not float since
    ///  we transmit integers only to the Collector Node.
    ///  For temperature, the Sensor Key is `t` for raw temperature (integer, from 0 to 4095).
    ///  The message will be enqueued for transmission by the CoAP / OIC 
    ///  Background Task so this function will return without waiting for the message 
    ///  to be transmitted.  Return 0 if successful, `SYS_EAGAIN` if network is not ready yet.
    ///  The CoAP payload needs to be very compact (under 32 bytes) so it will be encoded in CBOR like this:
    ///  `{ t: 2870 }`
    fn send_sensor_data_to_collector(sensor_val: &SensorValue,
                                     _node_id: &CStr) -> MynewtResult<()> {
        if unsafe { !NETWORK_IS_READY } {
            return Err(MynewtError::SYS_EAGAIN);
        }
        let rc = unsafe { sensor_network::init_collector_post() };
        if !rc {
            {
                ::core::panicking::panic(&("assertion failed: rc",
                                           "src/send_coap.rs", 269u32, 65u32))
            }
        };
        let _payload =
            {
                "begin cbor root";
                {
                    "begin cbor coap_root";
                    {
                        "begin oc_rep_start_root_object";
                        unsafe {
                            let encoder =
                                JSON_CONTEXT.encoder("root", "_map");
                            tinycbor::cbor_encoder_create_map(JSON_CONTEXT.global_encoder(),
                                                              encoder,
                                                              tinycbor::CborIndefiniteLength)
                        };
                        "end oc_rep_start_root_object";
                    };
                    {
                        " >>  >> sensor_val >> ,";
                        "--------------------";
                        {
                            "begin cbor coap_set_int_val , c : JSON_CONTEXT , val : sensor_val";
                            if let SensorValueType::Uint(val) = sensor_val.val
                                   {
                                "-- cinte c: JSON_CONTEXT, k: sensor_val.key, v: val";
                                let key_with_opt_null: &[u8] =
                                    sensor_val.key.to_bytes_optional_nul();
                                let value = val as i64;
                                "-------------------------------------------------------------";
                                (/*ERROR*/);
                                "-------------------------------------------------------------";
                                unsafe {
                                    let encoder =
                                        JSON_CONTEXT.encoder("JSON_CONTEXT",
                                                             "_map");
                                    tinycbor::cbor_encode_text_string(encoder,
                                                                      JSON_CONTEXT.key_to_cstr(key_with_opt_null),
                                                                      JSON_CONTEXT.cstr_len(key_with_opt_null));
                                    tinycbor::cbor_encode_int(encoder, value);
                                };
                            } else {
                                unsafe {
                                    JSON_CONTEXT.fail(json_context::JsonError::VALUE_NOT_UINT)
                                };
                            }
                            "end cbor coap_set_int_val";
                        };
                        "--------------------";
                    };
                    {
                        "begin oc_rep_end_root_object";
                        unsafe {
                            let encoder =
                                JSON_CONTEXT.encoder("root", "_map");
                            tinycbor::cbor_encoder_close_container(JSON_CONTEXT.global_encoder(),
                                                                   encoder)
                        };
                        "end oc_rep_end_root_object";
                    };
                    "end cbor coap_root";
                };
                "end cbor root";
                ()
            };
        let rc = unsafe { sensor_network::do_collector_post() };
        if !rc {
            {
                ::core::panicking::panic(&("assertion failed: rc",
                                           "src/send_coap.rs", 280u32, 63u32))
            }
        };
        console_print(b"NRF send to collector: rawtmp %d\n");
        Ok(())
    }
}
use core::panic::PanicInfo;
use cortex_m::asm::bkpt;
use mynewt::kernel::os;
use crate::base::*;
///  main() will be called at Mynewt startup. It replaces the C version of the main() function.
#[no_mangle]
extern "C" fn main() -> ! {
    unsafe { base::rust_sysinit() };
    unsafe { console_flush() };
    send_coap::start_network_task().expect("NET fail");
    listen_sensor::start_sensor_listener().expect("TMP fail");
    loop  { unsafe { os::os_eventq_run(os::os_eventq_dflt_get()) } }
}
///  This function is called on panic, like an assertion failure. We display the filename and line number and pause in the debugger. From https://os.phil-opp.com/freestanding-rust-binary/
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        let file = location.file();
        let line = location.line();
        console_print(b"panic at ");
        unsafe { console_buffer(file.as_ptr(), file.len() as u32) }
        console_print(b" line 0x");
        unsafe { console_printhex(line as u8) }
        console_print(b"\n");
        unsafe { console_flush() }
    } else {
        console_print(b"panic unknown loc\n");
        unsafe { console_flush() }
    }
    bkpt();
    loop  { }
}
