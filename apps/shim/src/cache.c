#include "cache.h"
#include "config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <utime.h>
#include <errno.h>

/* --- Helpers ------------------------------------------------------------ */

static void sha1_hex(const char *s, char out[41])
{
    unsigned long h1 = 5381, h2 = 0x811c9dc5;
    for (const unsigned char *p = (const unsigned char *)s; *p; p++) {
        h1 = ((h1 << 5) + h1) ^ *p;
        h2 = (h2 ^ *p) * 0x01000193;
    }
    snprintf(out, 41, "%016lx%016lx00000000", h1, h2);
}

static int mkdirs(const char *path)
{
    char tmp[4096];
    snprintf(tmp, sizeof(tmp), "%s", path);
    for (char *p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            mkdir(tmp, 0700);
            *p = '/';
        }
    }
    return mkdir(tmp, 0700) == 0 || errno == EEXIST ? 0 : -1;
}

/* --- API ---------------------------------------------------------------- */

int cache_get_path(const char *remote_path, char *out_path, size_t out_len)
{
    const httpfs_config_t *cfg = httpfs_get_config();
    char hex[41];
    sha1_hex(remote_path, hex);

    /* 2 niveaux de sous-répertoires pour éviter de saturer un répertoire */
    char dir[4096];
    snprintf(dir, sizeof(dir), "%s/%.2s/%.2s", cfg->cache_dir, hex, hex + 2);
    if (mkdirs(dir) < 0 && errno != EEXIST) return -1;

    snprintf(out_path, out_len, "%s/%s", dir, hex);
    return 0;
}

void cache_touch(const char *cache_path)
{
    /* Mise à jour de atime — utilisé par le GC pour le LRU */
    struct utimbuf ut = { .actime = time(NULL), .modtime = time(NULL) };
    utime(cache_path, &ut);
}

int cache_is_valid(const char *cache_path)
{
    const httpfs_config_t *cfg = httpfs_get_config();
    struct stat st;
    if (stat(cache_path, &st) != 0) return 0;
    return (time(NULL) - st.st_mtime) < cfg->ttl;
}

int cache_commit(const char *src_tmp, const char *cache_path)
{
    /* rename() est atomique sur le même FS — pas de fichier partiel visible */
    if (rename(src_tmp, cache_path) != 0) {
        perror("httpfs: cache_commit rename");
        return -1;
    }
    return 0;
}
