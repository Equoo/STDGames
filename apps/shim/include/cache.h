#pragma once
#include <stddef.h>
#include <time.h>

/*
 * Retourne le chemin local en cache pour un chemin distant.
 * Crée les répertoires intermédiaires si nécessaire.
 * out_path doit avoir au moins PATH_MAX octets.
 */
int cache_get_path(const char *remote_path, char *out_path, size_t out_len);

/*
 * Marque un fichier en cache comme récemment utilisé (met à jour atime).
 */
void cache_touch(const char *cache_path);

/*
 * Vérifie si le fichier est présent et non-expiré.
 * Retourne 1 si valide, 0 sinon.
 */
int cache_is_valid(const char *cache_path);

/*
 * Enregistre un fichier téléchargé dans le cache (déplacement atomique).
 * src_tmp est un fichier temporaire que cette fonction consomme.
 */
int cache_commit(const char *src_tmp, const char *cache_path);
