/* A C extension for egui-workbench. Build with build.sh; no WASI. */
typedef unsigned int u32;

__attribute__((import_module("workbench"), import_name("host_register")))
extern int host_register(int kind, const char *ptr, int len);
__attribute__((import_module("workbench"), import_name("host_log")))
extern void host_log(const char *ptr, int len);
__attribute__((import_module("workbench"), import_name("host_selection")))
extern void host_selection(const char *ptr, int len);

static const char command[] = "{\"id\":\"hello.run\",\"title\":\"Hello from C\"}";
static const char panel[] = "{\"id\":\"hello.panel\",\"title\":\"Hello panel\"}";
static const char viewer[] = "{\"id\":\"hello.viewer\",\"extensions\":[\"hello\"]}";

__attribute__((export_name("init"))) int init(void) {
    host_register(0, command, sizeof(command) - 1);
    host_register(1, panel, sizeof(panel) - 1);
    host_register(2, viewer, sizeof(viewer) - 1);
    host_selection("hello", 5);
    host_log("init", 4);
    return 0;
}

__attribute__((export_name("on_command"))) int on_command(const char *ptr, int len) {
    host_log(ptr, len);
    return 0;
}
