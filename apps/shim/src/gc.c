#include "cache.h"
#include "config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>
#include <time.h>
#include <pthread.h>

/* Entrée pour le tri LRU */
typedef struct {
    char   path[4096];
    time_t atime;
    off_t  size;
} cache_entry_t;

static int cmp_atime(const void *a, const void *b)
{
    return (int)(((cache_entry_t *)a)->atime - ((cache_entry_t *)b)->atime);
}

/* Parcours récursif du cache — remplit entries[], retourne le nb trouvé */
static size_t scan_cache(const char *dir,
                         cache_entry_t *entries, size_t cap,
                         off_t *total_bytes)
{
    DIR *d = opendir(dir);
    if (!d) return 0;

    struct dirent *ent;
    size_t count = 0;
    while ((ent = readdir(d)) != NULL && count < cap) {
        if (ent->d_name[0] == '.') continue;

        char path[4096];
        snprintf(path, sizeof(path), "%s/%s", dir, ent->d_name);

        struct stat st;
        if (stat(path, &st) != 0) continue;

        if (S_ISDIR(st.st_mode)) {
            count += scan_cache(path, entries + count, cap - count, total_bytes);
        } else {
            strncpy(entries[count].path, path, sizeof(entries[count].path) - 1);
            entries[count].atime = st.st_atime;
            entries[count].size  = st.st_size;
            *total_bytes += st.st_size;
            count++;
        }
    }
    closedir(d);
    return count;
}

static void gc_run(void)
{
    const httpfs_config_t *cfg = httpfs_get_config();
    time_t now = time(NULL);

    /* Allocation dynamique pour éviter un gros stack */
    size_t cap = 65536;
    cache_entry_t *entries = malloc(cap * sizeof(cache_entry_t));
    if (!entries) return;

    off_t total = 0;
    size_t count = scan_cache(cfg->cache_dir, entries, cap, &total);

    int removed = 0;

    /* 1. Supprime les fichiers expirés (TTL) */
    for (size_t i = 0; i < count; i++) {
        if ((now - entries[i].atime) > cfg->ttl) {
            if (unlink(entries[i].path) == 0) {
                total -= entries[i].size;
                entries[i].size = 0;   /* marque supprimé */
                removed++;
            }
        }
    }

    /* 2. Si encore trop grand → LRU eviction */
    if (total > cfg->max_cache_bytes) {
        qsort(entries, count, sizeof(cache_entry_t), cmp_atime);
        for (size_t i = 0; i < count && total > cfg->max_cache_bytes; i++) {
            if (entries[i].size == 0) continue;
            if (unlink(entries[i].path) == 0) {
                total -= entries[i].size;
                removed++;
            }
        }
    }

    if (cfg->verbose && removed > 0)
        fprintf(stderr, "[httpfs] gc: removed %d files, cache=%.1f MB\n",
                removed, (double)total / (1024*1024));

    free(entries);
}

static void *gc_thread(void *arg)
{
    (void)arg;
    const httpfs_config_t *cfg = httpfs_get_config();
    while (1) {
        sleep((unsigned)cfg->gc_interval);
        gc_run();
    }
    return NULL;
}

void gc_start(void)
{
    pthread_t tid;
    pthread_attr_t attr;
    pthread_attr_init(&attr);
    pthread_attr_setdetachstate(&attr, PTHREAD_CREATE_DETACHED);
    pthread_create(&tid, &attr, gc_thread, NULL);
    pthread_attr_destroy(&attr);
}
