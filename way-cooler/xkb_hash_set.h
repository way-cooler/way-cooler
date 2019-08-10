#include <assert.h>
#include <stdbool.h>

#include <xkbcommon/xkbcommon.h>

#ifndef WC_XKB_HASH_SET_H
#define WC_XKB_HASH_SET_H

struct hash_entry {
	xkb_mod_mask_t mod_mask;
	bool present;
};

struct xkb_hash_set {
	struct hash_entry set[XKB_KEY_VoidSymbol];
};

void xkb_hash_set_clear(struct xkb_hash_set *hash_set);

void xkb_hash_set_add_entry(
		struct xkb_hash_set *hash_set, uint32_t key, xkb_mod_mask_t mask);

bool xkb_hash_set_get_entry(
		struct xkb_hash_set *hash_set, uint32_t key, xkb_mod_mask_t *out_mask);

#endif  // WC_XKB_HASH_SET_H
