/* tapline — install Steam apps, dedicated servers and Workshop content.
 *
 * Hand-written rather than generated, for the same reason the protobuf output
 * is committed: a header is part of the interface, and a consumer should not
 * need a build-time tool to obtain one. It is checked against the Rust
 * signatures by a test in `tests/header.rs`, so drift is a test failure rather
 * than a mystery segfault in someone else's language.
 *
 * Threading: every function here is safe to call from any thread. A given
 * TaplineJob must not be used after tapline_job_free.
 *
 * Memory: the caller owns every buffer. The only thing this library allocates
 * on the caller's behalf is TaplineJob, released by tapline_job_free.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

#ifndef TAPLINE_H
#define TAPLINE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* An event was written to the buffer. */
#define TAPLINE_OK 0
/* The timeout elapsed with no event. Not an error: call again. */
#define TAPLINE_TIMEOUT 1
/* The job is over and no further events will arrive. */
#define TAPLINE_DONE 2
/* The buffer was too small. The needed length is in out_len and the event is
 * kept, so calling again with a bigger buffer returns it rather than the one
 * after it. */
#define TAPLINE_BUFFER_TOO_SMALL -1
/* An argument was unusable: a null pointer, or a string that is not UTF-8. */
#define TAPLINE_BAD_ARGUMENT -2

/* A running job. Opaque. */
typedef struct TaplineJob TaplineJob;

/* Installs or updates an app.
 *
 * dir         where to install; created if absent
 * branch      NULL for "public"
 * concurrency chunks in flight; 0 for the default (64)
 * os          0 host, 1 linux, 2 windows, 3 macos
 * validate    non-zero re-downloads even if the record says it is current
 * include_dlc non-zero installs DLC depots too
 * file_modes  0 matches steamcmd (0755 on everything), 1 uses the manifest's
 * extensions  comma-separated built-in names, or NULL; see below
 *
 * Returns TAPLINE_OK and writes a job to *out, or a negative code.
 */
int32_t tapline_install(uint32_t app_id,
                        const char *dir,
                        const char *branch,
                        uint32_t concurrency,
                        uint8_t os,
                        uint8_t validate,
                        uint8_t include_dlc,
                        uint8_t file_modes,
                        const char *extensions,
                        TaplineJob **out);

/* Works out what an install would cost without fetching any content.
 * Emits exactly one "planned" event, then ends. */
int32_t tapline_plan(uint32_t app_id,
                     const char *dir,
                     const char *branch,
                     uint8_t os,
                     uint8_t include_dlc,
                     TaplineJob **out);

/* Downloads one Workshop item. item_id is a published file id.
 *
 * flat non-zero writes the item's files straight into dir; zero uses
 * steamcmd's steamapps/workshop/content/<app>/<item>/ layout. A Garry's Mod
 * addon belongs in garrysmod/addons, which is what flat is for.
 *
 * extensions is a comma-separated list of built-in names, or NULL. Known:
 * "gmad" unpacks a .gma beside itself, "gmad-zip" converts it to a .zip,
 * "gmad-zip-stored" does so without deflating, and a trailing "!" on either
 * gmad name deletes the original once it has been processed. An unknown name
 * is an error rather than a no-op.
 *
 * stream non-zero unpacks a Garry's Mod addon as it downloads, without ever
 * writing the .gma. It implies the flat layout and ignores extensions, because
 * the archive those would act on never exists. */
int32_t tapline_workshop_download(uint32_t app_id,
                                  uint64_t item_id,
                                  const char *dir,
                                  uint32_t concurrency,
                                  uint8_t flat,
                                  const char *extensions,
                                  uint8_t stream,
                                  TaplineJob **out);

/* Waits for the next event and writes it to buf as UTF-8 JSON.
 *
 * timeout_ms of 0 polls without blocking, which is what a runtime with no
 * async FFI should use. Anything else blocks the calling thread for up to that
 * long, which is what a runtime that can move the call off its event loop
 * should use.
 *
 * Returns TAPLINE_OK with the length in out_len, TAPLINE_TIMEOUT, TAPLINE_DONE,
 * or TAPLINE_BUFFER_TOO_SMALL with the needed length in out_len.
 */
int32_t tapline_job_next(TaplineJob *job,
                         uint32_t timeout_ms,
                         uint8_t *buf,
                         size_t cap,
                         size_t *out_len);

/* Stops a job. Whatever is already on disk stays there, and a later install
 * resumes from it rather than starting over. */
void tapline_job_cancel(TaplineJob *job);

/* Frees a job, cancelling it first if it is still running. Null is allowed. */
void tapline_job_free(TaplineJob *job);

/* Writes the last error on this thread into buf. Only meaningful immediately
 * after a call that returned a negative code. Pass NULL to query the length. */
int32_t tapline_last_error(uint8_t *buf, size_t cap, size_t *out_len);

/* Sets the total chunks in flight across every job in this process.
 *
 * Downloads started from one process share one budget: two at 64 each is
 * measurably slower than two splitting 64, because the throughput curve turns
 * over after 64. Sharing also reuses warm connections between them.
 *
 * Must be called before the first job starts. Afterwards the budget is fixed
 * and this returns TAPLINE_BAD_ARGUMENT. 0 restores the default. */
int32_t tapline_set_total_concurrency(uint32_t chunks);

/* The total chunks in flight allowed across this process. */
uint32_t tapline_total_concurrency(void);

/* How much of that budget is free right now. Moves as you read it. */
uint32_t tapline_available_concurrency(void);

/* The library version, as a static NUL-terminated string. Do not free. */
const char *tapline_version(void);

#ifdef __cplusplus
}
#endif

#endif /* TAPLINE_H */
