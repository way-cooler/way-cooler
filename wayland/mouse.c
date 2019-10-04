/*
 * Copyright © 2019 Preston Carpenter <APragmaticPlace@gmail.com>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 */

#include "wayland/mouse.h"

#include <stdio.h>
#include <stdbool.h>

#include "globalconf.h"
#include "objects/button.h"

struct button_event {
    enum zway_cooler_mouse_state response_type;
    enum zway_cooler_mouse_button button;
};

static int xcb_conversion(enum zway_cooler_mouse_state state)
{
    return state == ZWAY_COOLER_MOUSE_STATE_RELEASE
            ? XCB_BUTTON_PRESS
            : XCB_BUTTON_RELEASE;
}

static bool
button_match(struct button_event *ev, button_t *b, void *data)
{
    printf("comparing %d and %d\n", xcb_conversion(ev->button), b->button);
    return (!b->button || xcb_conversion(ev->button) == b->button);
}

DO_EVENT_HOOK_CALLBACK(button, struct button_event, ZWAY_COOLER_MOUSE_STATE,
        button_array_t, button_match);

static void on_button(void *data,
        struct zway_cooler_mouse *zway_cooler_mouse,
        uint32_t time,
        uint32_t button,
        uint32_t state,
        int32_t x,
        int32_t y)
{
    lua_State *L = globalconf_get_lua_State();
    struct button_event event = {
        .response_type = state,
        .button = button,
    };
    event_button_callback(&event, &globalconf.buttons, L, 0, 0, NULL);
}

static void on_scroll(void *data,
        struct zway_cooler_mouse *zway_cooler_mouse,
        uint32_t time,
        uint32_t scroll,
        int32_t x,
        int32_t y)
{
    printf("scroll %d @ (%d , %d)\n", scroll, x, y);
}

static void on_move(void *data,
        struct zway_cooler_mouse *zway_cooler_mouse,
        uint32_t time,
        int32_t x,
        int32_t y)
{
    printf("move @ (%d , %d)\n", x, y);
}

struct zway_cooler_mouse_listener mouse_listener =
{
    .button = on_button,
    .scroll = on_scroll,
    .move = on_move,
};

// vim: filetype=c:expandtab:shiftwidth=4:tabstop=8:softtabstop=4:textwidth=80
