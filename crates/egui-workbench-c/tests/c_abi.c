#include <stdio.h>
#include <string.h>
#include "egui_workbench.h"

int main(void) {
    const char *spec = "{\"version\":1,\"widgets\":[{\"kind\":\"label\",\"text\":\"before\",\"size\":13.0,\"tone\":\"default\",\"strong\":false}]}";
    WbHost *host = wb_host_new(spec);
    if (!host) {
        fprintf(stderr, "new: %s\n", wb_last_error());
        return 1;
    }
    if (wb_host_apply_patch(host, "{\"op\":\"set\",\"path\":\"/widgets/0/text\",\"value\":\"after\"}") != 0) {
        fprintf(stderr, "patch: %s\n", wb_last_error());
        return 1;
    }
    char *current = wb_host_spec(host);
    if (!current || !strstr(current, "after")) {
        fprintf(stderr, "spec: %s\n", current ? current : wb_last_error());
        return 1;
    }
    wb_string_free(current);
    char *events = wb_host_frame(host);
    if (!events || strcmp(events, "[]") != 0) {
        fprintf(stderr, "frame: %s\n", events ? events : wb_last_error());
        return 1;
    }
    wb_string_free(events);
    char *schema = wb_host_schema();
    if (!schema || !strstr(schema, "HostSpec")) {
        fprintf(stderr, "schema\n");
        return 1;
    }
    wb_string_free(schema);
    if (strcmp(wb_host_version(), "0.13.0") != 0) {
        fprintf(stderr, "version: %s\n", wb_host_version());
        return 1;
    }
    wb_host_free(host);
    puts("c abi ok");
    return 0;
}
