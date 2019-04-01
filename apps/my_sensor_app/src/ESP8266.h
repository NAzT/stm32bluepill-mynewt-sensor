//  Ported from https://os.mbed.com/teams/ESP8266/code/esp8266-driver/file/6946b0b9e323/ESP8266/ESP8266.h/
/* ESP8266Interface Example
 * Copyright (c) 2015 ARM Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef ESP8266_H
#define ESP8266_H

#include "wifi.h"
#include "ATParser.h"

/** ESP8266Interface class.
    This is an interface to a ESP8266 radio.
 */
class ESP8266
{
public:
    ESP8266(bool debug=false);

    void configure(int uart);

    /**
    * Startup the ESP8266
    *
    * @param mode mode of WIFI 1-client, 2-host, 3-both
    * @return true only if ESP8266 was setup correctly
    */
    bool startup(int mode);

    /**
    * Reset ESP8266
    *
    * @return true only if ESP8266 resets successfully
    */
    bool reset(void);

    /**
    * Enable/Disable DHCP
    *
    * @param enabled DHCP enabled when true
    * @param mode mode of DHCP 0-softAP, 1-station, 2-both
    * @return true only if ESP8266 enables/disables DHCP successfully
    */
    bool dhcp(bool enabled, int mode);

    /**
    * Connect ESP8266 to AP
    *
    * @param ap the name of the AP
    * @param passPhrase the password of AP
    * @return true only if ESP8266 is connected successfully
    */
    bool connect(const char *ap, const char *passPhrase);

    /**
    * Disconnect ESP8266 from AP
    *
    * @return true only if ESP8266 is disconnected successfully
    */
    bool disconnect(void);

    /**
    * Get the IP address of ESP8266
    *
    * @return null-teriminated IP address or null if no IP address is assigned
    */
    const char *getIPAddress(void);

    /**
    * Get the MAC address of ESP8266
    *
    * @return null-terminated MAC address or null if no MAC address is assigned
    */
    const char *getMACAddress(void);

     /** Get the local gateway
     *
     *  @return         Null-terminated representation of the local gateway
     *                  or null if no network mask has been recieved
     */
    const char *getGateway();

    /** Get the local network mask
     *
     *  @return         Null-terminated representation of the local network mask 
     *                  or null if no network mask has been recieved
     */
    const char *getNetmask();

    /* Return RSSI for active connection
     *
     * @return      Measured RSSI
     */
    int8_t getRSSI();

    /**
    * Check if ESP8266 is conenected
    *
    * @return true only if the chip has an IP address
    */
    bool isConnected(void);

    /** Scan for available networks
     *
     * @param  ap    Pointer to allocated array to store discovered AP
     * @param  limit Size of allocated @a res array, or 0 to only count available AP
     * @return       Number of entries in @a res, or if @a count was 0 number of available networks, negative on error
     *               see @a nsapi_error
     */
    int scan(nsapi_wifi_ap_t *res, unsigned limit);

    /**
    * Open a socketed connection
    *
    * @param type the type of socket to open "UDP" or "TCP"
    * @param id id to give the new socket, valid 0-4
    * @param port port to open connection with
    * @param addr the IP address of the destination
    * @return true only if socket opened successfully
    */
    bool open(const char *type, int id, const char* addr, int port);

    /**
    * Sends data to an open socket
    *
    * @param id id of socket to send to
    * @param data data to be sent
    * @param amount amount of data to be sent - max 1024
    * @return true only if data sent successfully
    */
    bool send(int id, const void *data, uint32_t amount);

    /**
    * Receives data from an open socket
    *
    * @param id id to receive from
    * @param data placeholder for returned information
    * @param amount number of bytes to be received
    * @return the number of bytes received
    */
    int32_t recv(int id, void *data, uint32_t amount);

    /**
    * Closes a socket
    *
    * @param id id of socket to close, valid only 0-4
    * @return true only if socket is closed successfully
    */
    bool close(int id);

    /**
    * Allows timeout to be changed between commands
    *
    * @param timeout_ms timeout of the connection
    */
    void setTimeout(uint32_t timeout_ms);

    /**
    * Checks if data is available
    */
    bool readable();

    /**
    * Checks if data can be written
    */
    bool writeable();

    /**
    * Attach a function to call whenever network state has changed
    *
    * @param func A pointer to a void function, or 0 to set as none
    */
    void attach(void (*func)(void *), void *arg);

    void _packet_handler();
    int _uart;
private:
    BufferedSerial _serial;
    ATParser _parser;

    struct packet {
        struct packet *next;
        int id;
        uint32_t len;
        // data follows
    } *_packets, **_packets_end;
    bool recv_ap(nsapi_wifi_ap_t *ap);

    char _ip_buffer[16];
    char _gateway_buffer[16];
    char _netmask_buffer[16];
    char _mac_buffer[18];
};

#endif
