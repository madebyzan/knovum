import { shallowRef } from "vue";

export interface AudioInputDeviceOption {
  deviceId: string;
  label: string;
}

async function listAudioInputDevices(): Promise<AudioInputDeviceOption[]> {
  if (!globalThis.navigator?.mediaDevices?.enumerateDevices) return [];
  try {
    const devices = await navigator.mediaDevices.enumerateDevices();
    return devices
      .filter((device) => device.kind === "audioinput" && device.deviceId)
      .map((device) => ({ deviceId: device.deviceId, label: device.label.trim() }));
  } catch {
    return [];
  }
}

export function useAudioInputDevices() {
  const inputDevices = shallowRef<AudioInputDeviceOption[]>([]);
  const selectedDeviceId = shallowRef("");

  async function refresh() {
    const devices = await listAudioInputDevices();
    inputDevices.value = devices;
    if (selectedDeviceId.value && !devices.some((device) => device.deviceId === selectedDeviceId.value)) {
      selectedDeviceId.value = "";
    }
  }

  return { inputDevices, selectedDeviceId, refresh };
}
