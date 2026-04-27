#pragma once
#include <stddef.h>

/*
 * Télécharge une URL dans dest_path (fichier temporaire).
 * Supporte les Range requests pour des fichiers partiels.
 * Retourne 0 en cas de succès, -1 en erreur.
 */
int fetch_url(const char *url, const char *dest_tmp);

/*
 * Construit l'URL complète depuis un chemin intercepté.
 * out_url doit avoir au moins 4096 octets.
 */
void fetch_build_url(const char *remote_path, char *out_url, size_t out_len);
