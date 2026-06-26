import { nextTick, type Ref } from 'vue';

// 检查描述元素是否溢出（用于显示"查看完整介绍"按钮）
// 三个详情页（PlaylistDetail / AlbumDetail / ArtistDetail）共用
export function checkOverflow(
  elRef: Ref<HTMLElement | null>,
  overflowRef: Ref<boolean>,
  tolerance = 2,
) {
  nextTick(() => {
    if (elRef.value) {
      overflowRef.value = elRef.value.scrollHeight > elRef.value.clientHeight + tolerance;
    }
  });
}
