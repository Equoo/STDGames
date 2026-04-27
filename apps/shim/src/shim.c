/*
 * httpfs_preload — shim LD_PRELOAD qui redirige les accès fichiers
 * vers un cache local peuplé à la demande par HTTP.
 *
 * Appels interceptés :
 *   open / open64 / openat / openat64
 *   fopen / fopen64 / freopen
 *   stat / lstat / stat64 / lstat64 / fstatat
 *   access / faccessat
 *   opendir
 */

#include <dirent.h>
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <pthread.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#include <sys/stat.h>

#include "cache.h"
#include "config.h"
#include "fetch.h"

static httpfs_config_t g_config;
static int g_config_loaded = 0;

static long env_long(const char *name, long def) {
	const char *v = getenv(name);
	return v ? atol(v) : def;
}

const httpfs_config_t *httpfs_get_config(void) {
	if (g_config_loaded) return &g_config;

	g_config.prefix = getenv("HTTPFS_PREFIX") ?: DEFAULT_PREFIX;
	g_config.base_url = getenv("HTTPFS_BASE_URL") ?: DEFAULT_BASE_URL;
	g_config.cache_dir = getenv("HTTPFS_CACHE_DIR") ?: DEFAULT_CACHE_DIR;
	g_config.ttl = env_long("HTTPFS_TTL", DEFAULT_TTL);
	g_config.gc_interval = env_long("HTTPFS_GC_INTERVAL", DEFAULT_GC_INTERVAL);
	g_config.max_cache_bytes =
		env_long("HTTPFS_MAX_CACHE_MB", DEFAULT_MAX_CACHE_MB) * 1024L * 1024L;
	g_config.verbose = env_long("HTTPFS_VERBOSE", 0);

	g_config_loaded = 1;
	return &g_config;
}

/* ---- Déclarations des fonctions réelles -------------------------------- */

typedef int (*real_open_t)(const char *, int, ...);
typedef int (*real_openat_t)(int, const char *, int, ...);
typedef FILE *(*real_fopen_t)(const char *, const char *);
typedef FILE *(*real_freopen_t)(const char *, const char *, FILE *);
typedef int (*real_stat_t)(const char *, struct stat *);
typedef int (*real_lstat_t)(const char *, struct stat *);
typedef int (*real_fstatat_t)(int, const char *, struct stat *, int);
typedef int (*real_access_t)(const char *, int);
typedef DIR *(*real_opendir_t)(const char *);

static real_open_t r_open;
static real_openat_t r_openat;
static real_fopen_t r_fopen;
static real_freopen_t r_freopen;
static real_stat_t r_stat;
static real_lstat_t r_lstat;
static real_fstatat_t r_fstatat;
static real_access_t r_access;
static real_opendir_t r_opendir;

/* ---- Init (appelée une fois via __attribute__((constructor))) ---------- */

extern void gc_start(void); /* défini dans gc.c */

__attribute__((constructor)) static void httpfs_init(void) {
#define LOAD(sym)                                            \
	r_##sym = (real_##sym##_t)dlsym(RTLD_NEXT, #sym);        \
	if (!r_##sym) {                                          \
		fprintf(stderr, "[httpfs] dlsym " #sym " failed\n"); \
		abort();                                             \
	}
	LOAD(open)
	LOAD(openat)
	LOAD(fopen)
	LOAD(freopen)
	LOAD(stat)
	LOAD(lstat)
	LOAD(fstatat)
	LOAD(access)
	LOAD(opendir)
#undef LOAD

	/* S'assurer que le répertoire cache existe */
	const httpfs_config_t *cfg = httpfs_get_config();
	mkdir(cfg->cache_dir, 0700);

	/* Démarrer le thread GC */
	gc_start();

	if (cfg->verbose)
		fprintf(stderr, "[httpfs] init: prefix=%s base_url=%s cache=%s\n",
				cfg->prefix, cfg->base_url, cfg->cache_dir);
}

/* ---- Logique centrale : résoudre un chemin ----------------------------- */

/*
 * Si le chemin correspond au préfixe, retourne le chemin local en cache
 * (en téléchargeant si nécessaire).
 * Sinon, retourne NULL → passer au real_*.
 *
 * out_cache doit avoir PATH_MAX octets.
 * Retourne 1 si chemin intercepté et résolu, 0 sinon, -1 si erreur.
 */
static int resolve(const char *path, char *out_cache) {
	if (!path) return 0;

	const httpfs_config_t *cfg = httpfs_get_config();
	if (strncmp(path, cfg->prefix, strlen(cfg->prefix)) != 0)
		return 0; /* pas notre chemin */

	if (cache_get_path(path, out_cache, PATH_MAX) < 0) return -1;

	if (cache_is_valid(out_cache)) {
		cache_touch(out_cache);
		if (cfg->verbose) fprintf(stderr, "[httpfs] cache hit: %s\n", path);
		return 1;
	}

	/* Cache miss → téléchargement */
	char url[4096];
	fetch_build_url(path, url, sizeof(url));

	char tmp[PATH_MAX];
	snprintf(tmp, sizeof(tmp), "%s.tmp.XXXXXX", out_cache);
	int fd = mkstemp(tmp);
	if (fd < 0) return -1;
	close(fd);

	if (fetch_url(url, tmp) < 0) return -1;

	if (cache_commit(tmp, out_cache) < 0) return -1;

	return 1;
}

/* ---- Interceptions ----------------------------------------------------- */

int open(const char *path, int flags, ...) {
	mode_t mode = 0;
	if (flags & O_CREAT) {
		va_list ap;
		va_start(ap, flags);
		mode = va_arg(ap, mode_t);
		va_end(ap);
	}

	char cache[PATH_MAX];
	int r = resolve(path, cache);
	if (r == 1) return r_open(cache, flags, mode);
	if (r == -1) {
		errno = ENOENT;
		return -1;
	}
	return r_open(path, flags, mode);
}

int openat(int dirfd, const char *path, int flags, ...) {
	mode_t mode = 0;
	if (flags & O_CREAT) {
		va_list ap;
		va_start(ap, flags);
		mode = va_arg(ap, mode_t);
		va_end(ap);
	}

	char cache[PATH_MAX];
	int r = resolve(path, cache);
	if (r == 1) return r_openat(AT_FDCWD, cache, flags, mode);
	if (r == -1) {
		errno = ENOENT;
		return -1;
	}
	return r_openat(dirfd, path, flags, mode);
}

FILE *fopen(const char *path, const char *mode) {
	char cache[PATH_MAX];
	int r = resolve(path, cache);
	if (r == 1) return r_fopen(cache, mode);
	if (r == -1) {
		errno = ENOENT;
		return NULL;
	}
	return r_fopen(path, mode);
}

FILE *freopen(const char *path, const char *mode, FILE *stream) {
	char cache[PATH_MAX];
	int r = resolve(path, cache);
	if (r == 1) return r_freopen(cache, mode, stream);
	if (r == -1) {
		errno = ENOENT;
		return NULL;
	}
	return r_freopen(path, mode, stream);
}

int stat(const char *path, struct stat *st) {
	char cache[PATH_MAX];
	int r = resolve(path, cache);
	if (r == 1) return r_stat(cache, st);
	if (r == -1) {
		errno = ENOENT;
		return -1;
	}
	return r_stat(path, st);
}

int lstat(const char *path, struct stat *st) {
	/* Pour les symlinks sur les chemins distants, on délègue à stat */
	char cache[PATH_MAX];
	int r = resolve(path, cache);
	if (r == 1) return r_lstat(cache, st);
	if (r == -1) {
		errno = ENOENT;
		return -1;
	}
	return r_lstat(path, st);
}

int fstatat(int dirfd, const char *path, struct stat *st, int flags) {
	char cache[PATH_MAX];
	int r = resolve(path, cache);
	if (r == 1) return r_fstatat(AT_FDCWD, cache, st, flags);
	if (r == -1) {
		errno = ENOENT;
		return -1;
	}
	return r_fstatat(dirfd, path, st, flags);
}

int access(const char *path, int mode) {
	char cache[PATH_MAX];
	int r = resolve(path, cache);
	if (r == 1) return r_access(cache, mode);
	if (r == -1) {
		errno = ENOENT;
		return -1;
	}
	return r_access(path, mode);
}

int faccessat(int dirfd, const char *path, int mode, int flags) {
	(void)dirfd;
	(void)flags;
	/* Simplifié : on traite comme access() pour les chemins distants */
	return access(path, mode);
}

DIR *opendir(const char *path) {
	/* Les répertoires distants ne sont pas supportés — passthrough */
	return r_opendir(path);
}
