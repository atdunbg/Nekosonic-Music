/**
 * 歌曲相关类型定义
 */

/**
 * 歌曲信息
 */
export interface Song {
  /** 歌曲 ID */
  id: number;
  /** 歌曲名称 */
  name: string;
  /** 歌手列表 */
  ar: { id?: number; name: string }[];
  /** 专辑信息 */
  al: { id?: number; picUrl: string; name?: string };
  /** 时长（毫秒） */
  dt?: number;
  /** 本地文件路径（本地歌曲专用） */
  localPath?: string;
  /** 推荐算法标识（私人 FM 推荐） */
  alg?: string;
  /** 音频码率 */
  br?: number;
}

/**
 * 播放模式
 * - loop: 列表循环
 * - shuffle: 随机播放
 * - repeat-one: 单曲循环
 */
export type PlayMode = 'loop' | 'shuffle' | 'repeat-one';
