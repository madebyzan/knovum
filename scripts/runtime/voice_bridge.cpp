#include "engine/models/voxcpm2/loader.h"
#include "engine/framework/runtime/session.h"
#include "engine/framework/core/backend.h"
#include <algorithm>
#include <exception>
#include <memory>
#include <string>
#include <cstdint>

#ifdef _WIN32
#define KNOVUM_VOICE_API __declspec(dllexport)
#else
#define KNOVUM_VOICE_API __attribute__((visibility("default")))
#endif
using namespace engine;
using Callback = int (*)(const float *, size_t, int);
struct VoiceHandle {
    std::unique_ptr<models::voxcpm2::VoxCPM2LoadedModel> model;
    std::unique_ptr<runtime::IVoiceTaskSession> session;
};
static thread_local std::string failure;
extern "C" {
KNOVUM_VOICE_API const char *knovum_voice_error() { return failure.c_str(); }
KNOVUM_VOICE_API int knovum_voice_abi() { return 2; }
KNOVUM_VOICE_API int knovum_audio_devices(uint64_t minimum_free_bytes) {
    try {
        int device_flags = 1;
        for (const auto &device : core::list_backend_devices()) {
            if (device.index != 0 || (device.type != "GPU" && device.type != "IGPU")) continue;
            core::BackendType backend;
            int flag;
            if (device.backend == "Metal") { backend = core::BackendType::Metal; flag = 2; }
            else if (device.backend == "CUDA") { backend = core::BackendType::Cuda; flag = 4; }
            else if (device.backend == "Vulkan") { backend = core::BackendType::Vulkan; flag = 8; }
            else continue;
            const auto memory = core::query_backend_memory({backend, 0, 1});
            if (!memory.available) throw std::runtime_error("Compiled voice GPU memory statistics are unavailable");
            if (memory.free_bytes >= minimum_free_bytes) device_flags |= flag;
        }
        return device_flags;
    } catch (const std::exception &error) { failure = error.what(); return -1; }
}
KNOVUM_VOICE_API int knovum_voice_devices() {
    return knovum_audio_devices(8ULL * 1024 * 1024 * 1024);
}
KNOVUM_VOICE_API void *knovum_voice_open(const char *path, int device, int threads) {
    try {
        failure.clear();
        auto handle = std::make_unique<VoiceHandle>();
        handle->model = models::voxcpm2::load_voxcpm2_model(std::filesystem::u8path(path));
        runtime::SessionOptions options;
        core::BackendType backend;
        switch (device) {
            case 0: backend = core::BackendType::Cpu; break;
            case 1: backend = core::BackendType::Metal; break;
            case 2: backend = core::BackendType::Cuda; break;
            case 3: backend = core::BackendType::Vulkan; break;
            default: throw std::runtime_error("Unsupported frozen VoxCPM2 backend");
        }
        options.backend = {backend, 0, threads};
        handle->session = handle->model->create_task_session(
            {runtime::VoiceTaskKind::Tts, runtime::RunMode::Streaming}, options);
        return handle.release();
    } catch (const std::exception &error) { failure = error.what(); return nullptr; }
}
KNOVUM_VOICE_API void knovum_voice_close(void *opaque) { delete static_cast<VoiceHandle *>(opaque); }
KNOVUM_VOICE_API int knovum_voice_generate(void *opaque, const char *text, const float *reference, size_t reference_count,
                          const char *prompt_text, unsigned int seed, int max_tokens, Callback callback) {
    try {
        failure.clear();
        auto &session = dynamic_cast<runtime::IStreamingVoiceTaskSession &>(*static_cast<VoiceHandle *>(opaque)->session);
        runtime::TaskRequest request;
        request.text_input = runtime::Transcript{text, ""};
        request.options = {{"seed", std::to_string(seed)}, {"max_tokens", std::to_string(max_tokens)},
                           {"retry_badcase", "false"}};
        if (reference_count) {
            runtime::AudioBuffer audio{16000, 1, std::vector<float>(reference, reference + reference_count)};
            request.voice = runtime::VoiceCondition{};
            request.voice->speaker = runtime::VoiceReference{audio, std::nullopt};
            if (prompt_text && *prompt_text) {
                request.audio_input = audio;
                request.options["prompt_text"] = prompt_text;
            }
        }
        session.prepare(runtime::build_preparation_request(request));
        session.set_stream_event_sink([&](const runtime::StreamEvent &event) {
            for (const auto &named : event.named_audio_outputs) {
                if (named.audio.sample_rate != 48000 || named.audio.channels != 1)
                    throw std::runtime_error("VoxCPM2 output format differs from the frozen profile");
                constexpr size_t callback_samples = 16 * 1024;
                for (size_t offset = 0; offset < named.audio.samples.size(); offset += callback_samples) {
                    const auto count = std::min(callback_samples, named.audio.samples.size() - offset);
                    if (callback(named.audio.samples.data() + offset, count, named.audio.sample_rate))
                        throw std::runtime_error("voice-callback-stopped");
                }
            }
        });
        try { session.start_stream(request); }
        catch (...) { session.set_stream_event_sink(nullptr); session.reset(); throw; }
        session.set_stream_event_sink(nullptr);
        session.reset();
        return 0;
    } catch (const std::exception &error) { failure = error.what(); return 1; }
}
}
