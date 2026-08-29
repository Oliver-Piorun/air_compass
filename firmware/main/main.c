#include <stdio.h>

#include "dht.h"
#include "driver/gpio.h"

#include "esp_log.h"
#include "esp_wifi.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "mqtt_client.h"

#include "nvs_flash.h"

#include "config/secrets.h"

#define WIFI_CONNECTED_BIT BIT0

#define RED_LED_PIN GPIO_NUM_18
#define GREEN_LED_PIN GPIO_NUM_19
#define DHT_SENSOR_PIN GPIO_NUM_20

extern const uint8_t ca_crt_start[] asm("_binary_ca_crt_start");
extern const uint8_t client_crt_start[] asm("_binary_esp32_client_crt_start");
extern const uint8_t client_key_start[] asm("_binary_esp32_client_key_start");

static const char *TAG_WIFI = "WIFI";
static const char *TAG_MQTT = "MQTT";
static const char *TAG_DHT22 = "DHT22";

static EventGroupHandle_t wifi_event_group_handle;

static esp_mqtt_client_handle_t esp_mqtt_client_handle;

static void wifi_event_handler(void *event_handler_arg, esp_event_base_t event_base, int32_t event_id, void *event_data)
{
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START)
    {
        esp_wifi_connect();
    }
    else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED)
    {
        ESP_LOGW(TAG_WIFI, "Disconnected. Reconnecting...");
        esp_wifi_connect();
    }
    else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP)
    {
        ESP_LOGI(TAG_WIFI, "Connected");
        xEventGroupSetBits(
            wifi_event_group_handle,
            WIFI_CONNECTED_BIT // uxBitsToSet
        );
    }
}

static void mqtt_event_handler(void *event_handler_arg, esp_event_base_t event_base, int32_t event_id, void *event_data)
{
    esp_mqtt_event_handle_t esp_mqtt_event_handle = event_data;

    switch (esp_mqtt_event_handle->event_id)
    {
    case MQTT_EVENT_CONNECTED:
        ESP_LOGI(TAG_MQTT, "Connected to broker");
        break;
    case MQTT_EVENT_DISCONNECTED:
        ESP_LOGW(TAG_MQTT, "Disconnected from broker");
        break;
    case MQTT_EVENT_ERROR:
        ESP_LOGE(TAG_MQTT, "Error");
        break;
    default:
        break;
    }
}

void wifi_init(void)
{
    wifi_event_group_handle = xEventGroupCreate();

    // Initialize TCP/IP stack
    ESP_ERROR_CHECK(esp_netif_init());

    // Create default WiFi station
    ESP_ERROR_CHECK(esp_event_loop_create_default());

    esp_netif_create_default_wifi_sta();

    // Initialize the WiFi driver
    wifi_init_config_t wifi_init_config = WIFI_INIT_CONFIG_DEFAULT();

    ESP_ERROR_CHECK(esp_wifi_init(&wifi_init_config));
    ESP_ERROR_CHECK(esp_event_handler_register(
        WIFI_EVENT,
        ESP_EVENT_ANY_ID,
        &wifi_event_handler,
        NULL // event_handler_arg
        ));
    ESP_ERROR_CHECK(esp_event_handler_register(
        IP_EVENT,
        IP_EVENT_STA_GOT_IP,
        &wifi_event_handler,
        NULL // event_handler_arg
        ));

    wifi_config_t wifi_config = {
        .sta = {
            .ssid = WIFI_SSID,
            .password = WIFI_PASSWORD,
        },
    };

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());

    ESP_LOGI(TAG_WIFI, "Started");

    EventBits_t event_bits = xEventGroupWaitBits(
        wifi_event_group_handle,
        WIFI_CONNECTED_BIT, // uxBitsToWaitFor
        pdFALSE,            // xClearOnExit
        pdFALSE,            // xWaitForAllBits
        portMAX_DELAY       // xTicksToWait
    );

    if (event_bits & WIFI_CONNECTED_BIT)
    {
        ESP_LOGI(TAG_WIFI, "Ready");
    }
}

void mqtt_start(void)
{
    char mqtt_uri[128];
    snprintf(mqtt_uri, sizeof(mqtt_uri), "mqtts://%s:%d", MQTT_HOST, MQTT_PORT);

    esp_mqtt_client_config_t esp_mqtt_client_config = {
        .broker.address.uri = mqtt_uri,
        .broker.verification.certificate = (const char *)ca_crt_start,
        .credentials.authentication.certificate = (const char *)client_crt_start,
        .credentials.authentication.key = (const char *)client_key_start,
        .credentials.client_id = "esp32"
    };

    esp_mqtt_client_handle = esp_mqtt_client_init(&esp_mqtt_client_config);

    esp_mqtt_client_register_event(
        esp_mqtt_client_handle,
        ESP_EVENT_ANY_ID,
        mqtt_event_handler,
        NULL // event_handler_arg
    );

    esp_mqtt_client_start(esp_mqtt_client_handle);
}

void app_main(void)
{
    ESP_ERROR_CHECK(nvs_flash_init());

    wifi_init();
    mqtt_start();

    float temperature;
    float humidity;

    while (1)
    {
        esp_err_t esp_err = dht_read_float_data(
            DHT_TYPE_AM2301,
            DHT_SENSOR_PIN,
            &humidity,
            &temperature);

        if (esp_err == ESP_OK)
        {
            ESP_LOGI(TAG_DHT22, "Temperature: %.1f °C", temperature);
            ESP_LOGI(TAG_DHT22, "Humidity: %.1f %%", humidity);
        }
        else
        {
            ESP_LOGE(TAG_DHT22, "Read failed: %s", esp_err_to_name(esp_err));
        }

        // DHT22 max ~0.5 Hz sampling rate
        vTaskDelay(pdMS_TO_TICKS(2000));
    }
}
