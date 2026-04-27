#pragma once
#include <stddef.h>

/*
 * Toutes les variables d'environnement qui configurent httpfs_preload.
 *
 * HTTPFS_PREFIX    Préfixe des chemins à intercepter  (défaut: /remote/)
 * HTTPFS_BASE_URL  URL de base du serveur HTTP        (défaut: http://localhost:8080)
 * HTTPFS_CACHE_DIR Répertoire de cache local          (défaut: /tmp/httpfs_cache)
 * HTTPFS_TTL       TTL en secondes des fichiers cache (défaut: 3600)
 * HTTPFS_GC_INTERVAL Intervalle GC en secondes        (défaut: 300)
 * HTTPFS_MAX_CACHE_MB Taille max du cache en Mo       (défaut: 2048)
 * HTTPFS_VERBOSE   Active les logs debug (1/0)        (défaut: 0)
 */

#define DEFAULT_PREFIX        "/remote/"
#define DEFAULT_BASE_URL      "http://localhost:8080"
#define DEFAULT_CACHE_DIR     "/tmp/httpfs_cache"
#define DEFAULT_TTL           3600
#define DEFAULT_GC_INTERVAL   300
#define DEFAULT_MAX_CACHE_MB  20000

typedef struct {
    const char *prefix;
    const char *base_url;
    const char *cache_dir;
    long        ttl;
    long        gc_interval;
    long        max_cache_bytes;
    int         verbose;
} httpfs_config_t;

/* Singleton chargé une seule fois à l'init */
const httpfs_config_t *httpfs_get_config(void);
