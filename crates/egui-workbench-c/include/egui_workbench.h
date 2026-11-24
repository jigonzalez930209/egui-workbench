/* egui-workbench C ABI. MIT OR Apache-2.0. */
#ifndef EGUI_WORKBENCH_H
#define EGUI_WORKBENCH_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

typedef struct WbHost WbHost;

/* Create a host from a versioned HostSpec JSON document. Returns NULL on error. */
WbHost *wb_host_new(const char *spec_json);
/* Free a host. */
void wb_host_free(WbHost *host);

/* Apply a JSON-pointer patch. Returns 0 on success, -1 on error. */
int wb_host_apply_patch(WbHost *host, const char *patch_json);
/* The current spec as a newly allocated UTF-8 string. */
char *wb_host_spec(WbHost *host);
/* Drain events as a newly allocated JSON array. */
char *wb_host_poll_events(WbHost *host);
/* Run one headless frame and return its events as a JSON array. */
char *wb_host_frame(WbHost *host);
/* Open the native window on the calling thread. Returns 0 on success. */
int wb_host_run(WbHost *host);

/* The JSON Schema for HostSpec, newly allocated. */
char *wb_host_schema(void);
/* The ABI version. */
const char *wb_host_version(void);
/* Free a string returned by this library. */
void wb_string_free(char *text);
/* The last error for this thread. Valid until the next library call. */
const char *wb_last_error(void);

#ifdef __cplusplus
}
#endif

#endif
