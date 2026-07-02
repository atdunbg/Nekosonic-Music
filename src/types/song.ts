/**
 * 歌曲相关类型定义
 */

/**
 * 歌曲信息
 */
export interface Song {
  /** 歌曲 ID（网易云歌曲 ID；外部音源歌曲用占位负数） */
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
  /** 音频码率（bps，如 320000 表示 320kbps） */
  br?: number;
  /** 网易云付费标志：0=免费 1=VIP 4=数字专辑 8=低音质免费试听 */
  fee?: number;
  /** 当前可播放的最高码率（bps），来自 privilege.maxbr */
  maxbr?: number;
  /** 实际播放音源显示名（如 "netease"、"波点音乐"），由后端 fallback 返回 */
  source?: string;
  /** 外部音源歌曲专用：源内 ID（字符串，如 kuwo rid / kugou hash） */
  externalId?: string;
  /** 外部音源歌曲专用：源 ID（"bodian" / "kugou" / "kuwo" / "bilibili" / "custom-xxx"） */
  externalSourceId?: string;
  /** 外部音源歌曲专用：来源显示名（"波点音乐" / ...） */
  sourceLabel?: string;
}

/**
 * 播放模式
 * - loop: 列表循环
 * - shuffle: 随机播放
 * - repeat-one: 单曲循环
 */
export type PlayMode = 'loop' | 'shuffle' | 'repeat-one';
