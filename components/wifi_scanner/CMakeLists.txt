idf_component_register(
    SRCS "src/wifi_scanner.c"
    INCLUDE_DIRS "include"
    PRIV_REQUIRES esp_wifi lvgl nvs_flash rusty
)

# Declaring rusty in PRIV_REQUIRE in idf_component_register above just pulls in
# the headers. We need to declare the library dependency separately.
target_link_libraries(${COMPONENT_LIB} "rusty")
