#ifndef KNOVUM_VOICE_H
#define KNOVUM_VOICE_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
const char *knovum_voice_error(void);
int knovum_voice_abi(void);
int knovum_audio_devices(uint64_t minimum_free_bytes);
void *knovum_voice_open(const char *path,int device,int threads);
void knovum_voice_close(void *handle);
int knovum_voice_generate(void *handle,const char *text,const float *reference,size_t reference_count,
    const char *prompt_text,unsigned int seed,int max_tokens,
    int (*callback)(void *context,const float *samples,size_t count,int rate),void *context);
#ifdef __cplusplus
}
#endif
#endif
