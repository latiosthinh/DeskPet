// The C interface to DeskPet's shared pet host (macos/core/src/lib.rs), for the Mac app.
// It runs the same <id>.deskpet modules as the Windows app and draws them into the app's pixels.
// Everything but dp_update runs on the main thread. Returned strings stay valid until the next
// call that returns a string, so copy them straight away.

#pragma once
#include <stddef.h>
#include <stdint.h>

typedef struct DPHost DPHost;

/// Draws one line of text white on black and hands the coverage to dp_mask_set with `sink`.
/// font: 0 symbol glyphs (bold), 1 UI text (semibold). px: font size in device pixels.
typedef void (*DPTextFn)(const uint8_t *_Nonnull text, size_t len, uint8_t font, int32_t px, void *_Nonnull sink);
/// GETs `url`; on a 200 response of at most `limit` bytes, hands the body to dp_bytes_set with
/// `sink` and returns 1, otherwise returns 0. Called on a background thread.
typedef int32_t (*DPGetFn)(const char *_Nonnull url, size_t limit, void *_Nonnull sink);

typedef struct {
    double pet_x, min_pet_x, max_pet_x;
    double rect_x, rect_y, rect_w, rect_h;   // the pet's clickable rectangle, pen points, y up
    int32_t dragging;
    int32_t home_moved;                      // dragged to a new home: save it, then dp_home_saved
    int32_t can_remind;
} DPState;

// Making and freeing
DPHost *_Nonnull dp_new(const char *_Nonnull folder, const char *_Nullable extra, DPTextFn _Nonnull text);
void dp_free(DPHost *_Nonnull h);
void dp_mask_set(void *_Nonnull sink, int32_t w, int32_t h, const uint8_t *_Nonnull cov, size_t stride);

// The pet list
const char *_Nonnull dp_catalog_json(DPHost *_Nonnull h);
void dp_icon(DPHost *_Nonnull h, uint32_t index, int32_t size, uint32_t *_Nonnull out);
int32_t dp_reload(DPHost *_Nonnull h, const char *_Nullable show);
const char *_Nonnull dp_site(DPHost *_Nonnull h);

// Installing, removing and updating
int32_t dp_install(DPHost *_Nonnull h, const char *_Nonnull path);
int32_t dp_remove(DPHost *_Nonnull h, uint32_t index);
const char *_Nonnull dp_message(DPHost *_Nonnull h);
void dp_bytes_set(void *_Nonnull sink, const uint8_t *_Nullable bytes, size_t len);
int32_t dp_update(const char *_Nonnull folder, DPGetFn _Nonnull get);

// The pet on screen
int32_t dp_start(DPHost *_Nonnull h, uint32_t index, double now, uint64_t seed);
const char *_Nonnull dp_pet_json(DPHost *_Nonnull h);
int32_t dp_tick(DPHost *_Nonnull h, double dt, double now, double idle, double key_idle, double mouse_x, double mouse_y);
const char *_Nullable dp_failed(DPHost *_Nonnull h);
void dp_state(DPHost *_Nonnull h, DPState *_Nonnull out);
void dp_draw(DPHost *_Nonnull h, uint32_t *_Nonnull pixels, int32_t w, int32_t height, double scale);

// Commands
void dp_pen(DPHost *_Nonnull h, double width, double height, double range);
void dp_place_at(DPHost *_Nonnull h, double x);
void dp_say(DPHost *_Nonnull h, const char *_Nonnull text, double seconds);
void dp_show_pen(DPHost *_Nonnull h, double seconds);
void dp_trick(DPHost *_Nonnull h, uint32_t index);
void dp_surprise(DPHost *_Nonnull h);
void dp_hello(DPHost *_Nonnull h);
void dp_remind(DPHost *_Nonnull h, uint32_t which);   // 0 water, 1 lunch, 2 dinner, 3 stretch
void dp_begin_drag(DPHost *_Nonnull h);
void dp_end_drag(DPHost *_Nonnull h, int32_t moved);
int32_t dp_right_click(DPHost *_Nonnull h);           // 1 if the pet did a trick, 0: show the menu
void dp_stop_roaming(DPHost *_Nonnull h);
void dp_home_saved(DPHost *_Nonnull h);
void dp_demo(DPHost *_Nonnull h, uint32_t kind, double seconds);   // 0 walk, 1 type, 2 pastime, 3 sweep
