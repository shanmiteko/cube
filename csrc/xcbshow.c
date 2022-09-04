#include <string.h>
#include "xcbshow.h"

xcb_atom_t
get_atom(xcb_connection_t *conn,
         const char *name)
{
    xcb_atom_t atom;
    xcb_generic_error_t *error;
    xcb_intern_atom_cookie_t cookie;
    xcb_intern_atom_reply_t *reply;

    error = NULL;
    cookie = xcb_intern_atom(conn, 1, strlen(name), name);
    reply = xcb_intern_atom_reply(conn, cookie, &error);

    if (NULL != error)
    {
        free(reply);
        free(error);
        dieln("%s", "xcb_intern_atom failed");
    }

    atom = reply->atom;
    free(reply);

    return atom;
}

window_t *
create_window(uint16_t width,
              uint16_t height)
{
    debug_println("width: %d, height: %d", width, height);
    window_t *window = malloc(sizeof(window_t));

    if (xcb_connection_has_error(window->xcb_conn = xcb_connect(NULL, NULL)))
    {
        free(window);
        dieln("%s", "can't open display");
    }

    if (NULL == (window->xcb_screen = xcb_setup_roots_iterator(xcb_get_setup(window->xcb_conn)).data))
    {
        xcb_disconnect(window->xcb_conn);
        free(window);
        dieln("%s", "can't get default screen");
    }

    window->xcb_window = xcb_generate_id(window->xcb_conn);
    window->xcb_gc = xcb_generate_id(window->xcb_conn);

    xcb_create_window(
        window->xcb_conn, XCB_COPY_FROM_PARENT, window->xcb_window, window->xcb_screen->root,
        0, 0, width, height, 0, XCB_WINDOW_CLASS_INPUT_OUTPUT,
        window->xcb_screen->root_visual, XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK,
        (const uint32_t[]){
            window->xcb_screen->white_pixel,
            XCB_EVENT_MASK_EXPOSURE |
                XCB_EVENT_MASK_BUTTON_MOTION |
                XCB_EVENT_MASK_BUTTON_PRESS |
                XCB_EVENT_MASK_BUTTON_RELEASE |
                XCB_EVENT_MASK_KEY_PRESS});

    xcb_create_gc(window->xcb_conn, window->xcb_gc, window->xcb_window, 0, NULL);

    /* set WM_NAME */
    xcb_change_property(
        window->xcb_conn, XCB_PROP_MODE_REPLACE, window->xcb_window,
        XCB_ATOM_WM_NAME, XCB_ATOM_STRING, 8,
        sizeof("xcbshow") - 1, "xcbshow");

    /* add WM_DELETE_WINDOW to WM_PROTOCOLS */
    xcb_change_property(
        window->xcb_conn, XCB_PROP_MODE_REPLACE, window->xcb_window,
        get_atom(window->xcb_conn, "WM_PROTOCOLS"), XCB_ATOM_ATOM, 32, 1,
        (const xcb_atom_t[]){get_atom(window->xcb_conn, "WM_DELETE_WINDOW")});

    xcb_map_window(window->xcb_conn, window->xcb_window);
    xcb_flush(window->xcb_conn);

    return window;
}

void destroy_window(window_t *window)
{
    debug_println("window: %p", window);
    xcb_free_gc(window->xcb_conn, window->xcb_gc);
    xcb_destroy_window(window->xcb_conn, window->xcb_window);
    xcb_disconnect(window->xcb_conn);
    free(window);
}

image_t *
create_image(window_t *window,
             uint16_t width,
             uint16_t height)
{
    debug_println("window: %p, width: %d, height: %d", window, width, height);
    image_t *image = malloc(sizeof(image_t));
    if (NULL == image)
    {
        dieln("%s", "error while calling malloc, no memory available");
    }
    image->pixel_count = width * height;
    image->pixel = calloc(image->pixel_count, sizeof(uint32_t));
    if (NULL == image->pixel)
    {
        free(image);
        dieln("%s", "error while calling malloc, no memory available");
    }
    image->xcb_image = xcb_image_create_native(
        window->xcb_conn, width, height,
        XCB_IMAGE_FORMAT_Z_PIXMAP,
        window->xcb_screen->root_depth, image->pixel,
        sizeof(uint32_t) * image->pixel_count,
        (uint8_t *)(image->pixel));
    return image;
}

void show_image(window_t *window,
                image_t *image)
{
    debug_println("window: %p, image: %p", window, image);
    xcb_image_put(window->xcb_conn, window->xcb_window,
                  window->xcb_gc, image->xcb_image,
                  0, 0, 0);
    xcb_flush(window->xcb_conn);
}

void resize_image(window_t *window,
                  image_t *ori_image,
                  uint16_t width,
                  uint16_t height)
{
    debug_println("window: %p, ori_image: %p, width: %d, height: %d", window, ori_image, width, height);
    if (ori_image->xcb_image->width != width ||
        ori_image->xcb_image->height != height)
    {
        destroy_image(ori_image);
        ori_image = create_image(window, width, height);
    }
}

void update_image(image_t *ori_image,
                  const uint32_t *pixel_base)
{
    debug_println("ori_image: %p, pixel_base: %p, pixel_0: %x, pixel_1: %x", ori_image, pixel_base, pixel_base[0], pixel_base[1]);
    memcpy(ori_image->pixel, (uint32_t *)pixel_base, sizeof(uint32_t) * ori_image->pixel_count);
}

void destroy_image(image_t *image)
{
    debug_println("image: %p", image);
    xcb_image_destroy(image->xcb_image);
    free(image);
}

event_t *wait_for_event(window_t *window)
{
    debug_println("window: %p", window);
    xcb_generic_event_t *ev = xcb_wait_for_event(window->xcb_conn);
    event_t *event = malloc(sizeof(event_t));

    if (NULL == event)
    {
        dieln("%s", "error while calling malloc, no memory available");
    }
    switch (ev->response_type & ~0x80)
    {
    case XCB_CLIENT_MESSAGE:
    {
        if (((xcb_client_message_event_t *)ev)->data.data32[0] ==
            get_atom(window->xcb_conn, "WM_DELETE_WINDOW"))
        {
            event->kind = 1;
        }
        break;
    }
    case XCB_EXPOSE:
    {
        xcb_expose_event_t *eev = (xcb_expose_event_t *)ev;
        event->kind = 2;
        event->width = eev->width;
        event->height = eev->height;
        break;
    }
    case XCB_MOTION_NOTIFY:
    {
        xcb_motion_notify_event_t *mnev = (xcb_motion_notify_event_t *)ev;
        event->kind = 3;
        event->x = mnev->event_x;
        event->y = mnev->event_y;
        event->state = mnev->state;
        event->detail = mnev->detail;
        break;
    }
    case XCB_BUTTON_PRESS:
    {
        xcb_button_press_event_t *bpev = (xcb_button_press_event_t *)ev;
        event->kind = 4;
        event->x = bpev->event_x;
        event->y = bpev->event_y;
        event->state = bpev->state;
        event->detail = bpev->detail;
        break;
    }
    case XCB_BUTTON_RELEASE:
    {
        xcb_button_release_event_t *brev = (xcb_button_release_event_t *)ev;
        event->kind = 5;
        event->x = brev->event_x;
        event->y = brev->event_y;
        event->state = brev->state;
        event->detail = brev->detail;
        break;
    }
    case XCB_KEY_PRESS:
    {
        xcb_key_press_event_t *kpev = (xcb_key_press_event_t *)ev;
        event->kind = 6;
        event->x = kpev->event_x;
        event->y = kpev->event_y;
        event->state = kpev->state;
        event->detail = kpev->detail;
        break;
    }
    }

    free(ev);

    return event;
}

void destroy_event(event_t *event)
{
    debug_println("event: %p", event);
    free(event);
}
