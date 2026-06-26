import { invoke } from '@tauri-apps/api/core';

/**
 * 音频播放控制 API
 */
export const AudioApi = {
  /** 播放网络音频 */
  async playAudio(url: string): Promise<void> {
    return invoke('play_audio', { url });
  },

  /** 播放本地音频文件 */
  async playLocalAudio(path: string): Promise<void> {
    return invoke('play_local_audio', { path });
  },

  /** 暂停播放 */
  async pauseAudio(): Promise<void> {
    return invoke('pause_audio');
  },

  /** 恢复播放 */
  async resumeAudio(): Promise<void> {
    return invoke('resume_audio');
  },

  /** 停止播放 */
  async stopAudio(): Promise<void> {
    return invoke('stop_audio');
  },

  /** 跳转到指定位置（秒） */
  async seekAudio(time: number): Promise<void> {
    return invoke('seek_audio', { time });
  },

  /** 设置音量（0.0 - 1.0） */
  async setVolume(vol: number): Promise<void> {
    return invoke('set_volume', { vol });
  },

  /** 获取当前播放位置（秒） */
  async getAudioPosition(): Promise<number> {
    return invoke('get_audio_position');
  },

  /** 是否正在播放 */
  async isAudioPlaying(): Promise<boolean> {
    return invoke('is_audio_playing');
  },
};
