idf_component_register(
    SRCS "main.c"
    INCLUDE_DIRS "."
    PRIV_REQUIRES
        esp_wifi
        nvs_flash
    EMBED_TXTFILES
        certs/ca.crt
        certs/client.crt
        certs/client.key
)
