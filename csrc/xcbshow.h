#ifndef __XCBSHOW_H__
#define __XCBSHOW_H__

#include <stdlib.h>
#include <stdio.h>
#include <xcb/xcb_image.h>

#define LOG(stream, head, fmt, ...) \
    fprintf(stream, "%s %s: %s(): " fmt "\n", head, __FILE__, __func__, __VA_ARGS__)

#ifndef dieln
#define dieln(fmt, ...)                           \
    do                                            \
    {                                             \
        LOG(stderr, "[ERROR]", fmt, __VA_ARGS__); \
        exit(0);                                  \
    } while (0)
#endif

#if defined(DEBUG)
#define debug_println(fmt, ...)                   \
    do                                            \
    {                                             \
        LOG(stdout, "[DEBUG]", fmt, __VA_ARGS__); \
    } while (0)
#else
#define debug_println(fmt, ...)
#endif // DEBUG

typedef struct window_t
{
    xcb_connection_t *xcb_conn;
    xcb_screen_t *xcb_screen;
    xcb_window_t xcb_window;
    xcb_gcontext_t xcb_gc;
} window_t;

typedef struct image_t
{
    uint32_t pixel_count;
    uint32_t *pixel;
    xcb_image_t *xcb_image;
} image_t;

typedef struct event_t
{
    uint16_t width;
    uint16_t height;
    int16_t x;
    int16_t y;
    uint16_t state;
    uint8_t detail;
    uint8_t kind;
} event_t;

xcb_atom_t
get_atom(xcb_connection_t *conn, const char *name);

window_t *
create_window(uint16_t width,
              uint16_t height);

void destroy_window(window_t *window);

image_t *
create_image(window_t *window,
             uint16_t width,
             uint16_t height);

void show_image(window_t *window,
                image_t *image);

void resize_image(window_t *window,
                  image_t *ori_image,
                  uint16_t width,
                  uint16_t height);

void destroy_image(image_t *image);

event_t *wait_for_event(window_t *window);

void destroy_event(event_t *event);

#endif