#include "xkb_hash_set.h"

#include <string.h>

void xkb_hash_set_clear(struct xkb_hash_set *hash_set) {
	memset(&hash_set->set, 0, sizeof(hash_set->set));
}

void xkb_hash_set_add_entry(
		struct xkb_hash_set *hash_set, uint32_t key, xkb_mod_mask_t mask) {
	assert(key < sizeof(hash_set->set));

	struct hash_entry *entry = &hash_set->set[key];
	entry->present = true;
	entry->mod_mask = mask;
}

bool xkb_hash_set_get_entry(
		struct xkb_hash_set *hash_set, uint32_t key, xkb_mod_mask_t *out_mask) {
	assert(key < sizeof(hash_set->set));

	struct hash_entry *entry = &hash_set->set[key];
	if (entry->present) {
		if (out_mask != NULL) {
			*out_mask = entry->mod_mask;
		}
		return true;
	}
	return false;
}
