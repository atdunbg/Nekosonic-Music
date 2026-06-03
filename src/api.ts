import { invoke } from '@tauri-apps/api/core';

export namespace MusicApi {
  export async function getLoginStatus(): Promise<string> {
    return invoke('get_login_status');
  }

  export async function logout(): Promise<void> {
    return invoke('logout');
  }

  export async function getQrKey(): Promise<string> {
    return invoke('get_qr_key');
  }

  export async function checkQrStatus(key: string): Promise<string> {
    return invoke('check_qr_status', { query: { key } });
  }

  export async function likelist(uid: number): Promise<string> {
    return invoke('likelist', { uid });
  }

  export async function likeSong(id: number, like: boolean): Promise<void> {
    return invoke('like_song', { query: { id, like: like ? 'true' : 'false' } });
  }

  export async function userPlaylist(uid: number): Promise<string> {
    return invoke('user_playlist', { uid });
  }

  export async function getPlaylistDetail(id: number): Promise<string> {
    return invoke('get_playlist_detail', { id });
  }

  export async function playlistTrackAll(id: number): Promise<string> {
    return invoke('playlist_track_all', { query: { id } });
  }

  export async function playlistSubscribe(id: number, subscribe: boolean): Promise<void> {
    return invoke('playlist_subscribe', { query: { id, subscribe } });
  }

  export async function recommendResource(): Promise<string> {
    return invoke('recommend_resource');
  }

  export async function recommendSongs(): Promise<string> {
    return invoke('recommend_songs');
  }

  export async function getSongDetail(id: string): Promise<string> {
    return invoke('get_song_detail', { id });
  }

  export async function getSongUrl(query: { id: number; level: string; fm_mode?: boolean }): Promise<string> {
    return invoke('get_song_url', { query });
  }

  export async function getLyric(id: number): Promise<string> {
    return invoke('get_lyric', { id });
  }

  export async function searchSuggest(keyword: string): Promise<string> {
    return invoke('search_suggest', { query: { keyword } });
  }

  export async function getHotSearch(): Promise<string> {
    return invoke('get_hot_search');
  }

  export async function cloudsearch(query: { keyword: string; searchType: number; limit: number }): Promise<string> {
    return invoke('cloudsearch', { query });
  }

  export async function albumDetail(id: number): Promise<string> {
    return invoke('album_detail', { id });
  }

  export async function artistDetail(id: number): Promise<string> {
    return invoke('artist_detail', { id });
  }

  export async function artistSongs(query: { id: number; order: string; limit: number; offset: number }): Promise<string> {
    return invoke('artist_songs', { query });
  }

  export async function artistAlbum(id: number, limit: number, offset: number): Promise<string> {
    return invoke('artist_album', { id, limit, offset });
  }

  export async function artistDesc(id: number): Promise<string> {
    return invoke('artist_desc', { id });
  }

  export async function artistSub(id: number, sub: boolean): Promise<string> {
    return invoke('artist_sub', { query: { id, sub } });
  }

  export async function artistSublist(limit = 100, offset = 0): Promise<string> {
    return invoke('artist_sublist', { query: { limit, offset } });
  }

  export async function commentHot(query: { type: number; id: number; limit: number; offset: number }): Promise<string> {
    return invoke('comment_hot', { query });
  }

  export async function commentLike(query: { t: number; type: number; id: number; cid: number }): Promise<void> {
    return invoke('comment_like', { query });
  }

  export async function personalFm(): Promise<string> {
    return invoke('personal_fm');
  }

  export async function personalFmMode(query: { mode: string; subMode: string; limit: number }): Promise<string> {
    return invoke('personal_fm_mode', { query });
  }

  export async function fmTrash(id: number, time: number): Promise<void> {
    return invoke('fm_trash', { query: { id, time } });
  }

  export async function scrobble(query: { id: number; sourceid: string; time: number }): Promise<void> {
    return invoke('scrobble', { query });
  }

  // 云盘
  export async function userCloud(limit = 30, offset = 0): Promise<string> {
    return invoke('user_cloud', { limit, offset });
  }

  export async function userCloudDel(id: number): Promise<string> {
    return invoke('user_cloud_del', { id });
  }

  export async function cloudUpload(filePath: string): Promise<string> {
    return invoke('cloud_upload', { filePath });
  }
}

export namespace AudioApi {
  export async function playAudio(url: string): Promise<void> {
    return invoke('play_audio', { url });
  }

  export async function playLocalAudio(path: string): Promise<void> {
    return invoke('play_local_audio', { path });
  }

  export async function pauseAudio(): Promise<void> {
    return invoke('pause_audio');
  }

  export async function resumeAudio(): Promise<void> {
    return invoke('resume_audio');
  }

  export async function stopAudio(): Promise<void> {
    return invoke('stop_audio');
  }

  export async function seekAudio(time: number): Promise<void> {
    return invoke('seek_audio', { time });
  }

  export async function setVolume(vol: number): Promise<void> {
    return invoke('set_volume', { vol });
  }

  export async function getAudioPosition(): Promise<number> {
    return invoke('get_audio_position');
  }

  export async function isAudioPlaying(): Promise<boolean> {
    return invoke('is_audio_playing');
  }
}

export namespace DeviceApi {
  export async function getOutputDevices(): Promise<string[]> {
    return invoke('get_output_devices');
  }

  export async function setOutputDevice(device: string | null): Promise<void> {
    return invoke('set_output_device', { device });
  }
}

export namespace DownloadApi {
  export async function downloadSong(query: {
    id: number;
    name: string;
    artist: string;
    album: string | null;
    duration: number | null;
    coverUrl: string | null;
    level: string;
    downloadPath: string | null;
  }): Promise<void> {
    return invoke('download_song', { query });
  }

  export async function listLocalSongs(downloadPath: string | null): Promise<any[]> {
    return invoke('list_local_songs', { downloadPath });
  }

  export async function scanLocalFolders(paths: string[]): Promise<any[]> {
    return invoke('scan_local_folders', { paths });
  }

  export async function deleteLocalSong(query: { id: number; filename: string; downloadPath: string | null }): Promise<void> {
    return invoke('delete_local_song', { query });
  }

  export async function getDefaultDownloadPath(): Promise<string> {
    return invoke('get_default_download_path');
  }
}

export namespace AppApi {
  export function exitApp(): Promise<void> {
    return invoke('exit_app');
  }
}
