/**
 * API 统一入口
 *
 * 按业务域拆分到独立文件，此处重新组装为命名空间以保持向后兼容。
 * 新代码建议直接从具体域文件导入，例如：
 *   import { SongApi } from '../api/song';
 */
import { LoginApi } from './login';
import { SongApi } from './song';
import { PlaylistApi } from './playlist';
import { SearchApi } from './search';
import { AlbumApi } from './album';
import { ArtistApi } from './artist';
import { CommentApi } from './comment';
import { FmApi } from './fm';
import { CloudApi } from './cloud';
import { AudioApi } from './audio';
import { DeviceApi } from './device';
import { DownloadApi } from './download';
import { AppApi } from './app';
import { RecApi } from './rec';

export { LoginApi, SongApi, PlaylistApi, SearchApi, AlbumApi, ArtistApi, CommentApi, FmApi, CloudApi, AudioApi, DeviceApi, DownloadApi, AppApi, RecApi };

/**
 * 音乐业务 API（向后兼容命名空间）
 * @deprecated 建议直接使用具体域的 Api 对象，如 `SongApi`、`PlaylistApi` 等
 */
export const MusicApi = {
  // 登录
  getLoginStatus: LoginApi.getLoginStatus,
  logout: LoginApi.logout,
  getQrKey: LoginApi.getQrKey,
  checkQrStatus: LoginApi.checkQrStatus,

  // 歌曲
  getSongDetail: SongApi.getSongDetail,
  getSongUrl: SongApi.getSongUrl,
  getLyric: SongApi.getLyric,
  likelist: SongApi.likelist,
  likeSong: SongApi.likeSong,
  scrobble: SongApi.scrobble,

  // 歌单
  userPlaylist: PlaylistApi.userPlaylist,
  getPlaylistDetail: PlaylistApi.getPlaylistDetail,
  playlistTrackAll: PlaylistApi.playlistTrackAll,
  playlistSubscribe: PlaylistApi.playlistSubscribe,
  recommendResource: PlaylistApi.recommendResource,
  recommendSongs: PlaylistApi.recommendSongs,

  // 搜索
  searchSuggest: SearchApi.searchSuggest,
  getHotSearch: SearchApi.getHotSearch,
  cloudsearch: SearchApi.cloudsearch,

  // 专辑
  albumDetail: AlbumApi.albumDetail,

  // 歌手
  artistDetail: ArtistApi.artistDetail,
  artistSongs: ArtistApi.artistSongs,
  artistAlbum: ArtistApi.artistAlbum,
  artistDesc: ArtistApi.artistDesc,
  artistSub: ArtistApi.artistSub,
  artistSublist: ArtistApi.artistSublist,

  // 评论
  commentHot: CommentApi.commentHot,
  commentLike: CommentApi.commentLike,

  // 私人 FM
  personalFm: FmApi.personalFm,
  personalFmMode: FmApi.personalFmMode,
  fmTrash: FmApi.fmTrash,

  // 云盘
  userCloud: CloudApi.userCloud,
  userCloudDel: CloudApi.userCloudDel,
  cloudUpload: CloudApi.cloudUpload,

  // 推荐
  personalized: RecApi.personalized,
  personalizedNewsong: RecApi.personalizedNewsong,
  topArtists: RecApi.topArtists,
  topSong: RecApi.topSong,
  albumNewest: RecApi.albumNewest,
};
