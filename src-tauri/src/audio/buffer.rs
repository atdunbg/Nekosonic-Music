//! 流式音频缓冲区
//!
//! 提供线程安全的共享缓冲区与流式读取器，用于在下载线程与解码线程之间传递音频数据。

use ringbuf::{HeapCons, HeapProd, HeapRb, traits::Split};
use std::io::Read;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use symphonia::core::io::MediaSource;

/// 缓冲区内部状态，存储已下载的字节数据及完成/取消标志
pub(crate) struct BufferState {
    pub(crate) bytes: Vec<u8>,
    pub(crate) done: bool,
    pub(crate) cancelled: bool,
}

/// 线程安全的共享缓冲区，支持生产者写入和消费者读取的同步等待
pub struct SharedBuffer {
    state: Mutex<BufferState>,
    available: Condvar,
}

impl SharedBuffer {
    /// 创建新的空共享缓冲区
    pub fn new() -> Self {
        SharedBuffer {
            state: Mutex::new(BufferState {
                bytes: Vec::new(),
                done: false,
                cancelled: false,
            }),
            available: Condvar::new(),
        }
    }

    /// 向缓冲区追加写入一块数据，并通知等待的读取者
    pub fn write_chunk(&self, chunk: &[u8]) {
        let mut state = self.state.lock().unwrap();
        state.bytes.extend_from_slice(chunk);
        self.available.notify_all();
    }

    /// 标记缓冲区写入已完成，通知读取者不再有新数据
    pub fn mark_done(&self) {
        let mut state = self.state.lock().unwrap();
        state.done = true;
        self.available.notify_all();
    }

    /// 取消缓冲区，中断正在进行的读写操作
    pub fn cancel(&self) {
        let mut state = self.state.lock().unwrap();
        state.cancelled = true;
        self.available.notify_all();
    }

    /// 返回已缓冲的数据字节数
    pub fn len(&self) -> usize {
        self.state.lock().unwrap().bytes.len()
    }

    /// 检查缓冲区是否已标记为写入完成
    pub fn is_done(&self) -> bool {
        self.state.lock().unwrap().done
    }

    /// 检查缓冲区是否已被取消
    pub fn is_cancelled(&self) -> bool {
        self.state.lock().unwrap().cancelled
    }

    /// 获取内部状态锁（供 StreamingReader 使用）
    pub(crate) fn state_mutex(&self) -> &Mutex<BufferState> {
        &self.state
    }

    /// 获取内部条件变量（供 StreamingReader 使用）
    pub(crate) fn condvar(&self) -> &Condvar {
        &self.available
    }
}

/// 流式读取器，从共享缓冲区中按需读取数据，实现 `Read` 和 `Seek` trait
pub struct StreamingReader {
    buffer: Arc<SharedBuffer>,
    pos: usize,
}

impl StreamingReader {
    /// 创建新的流式读取器，绑定到指定的共享缓冲区
    pub fn new(buffer: Arc<SharedBuffer>) -> Self {
        StreamingReader { buffer, pos: 0 }
    }
}

impl Read for StreamingReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut state = self.buffer.state_mutex().lock().unwrap();
        loop {
            let available = state.bytes.len().saturating_sub(self.pos);
            if available > 0 {
                let to_read = std::cmp::min(buf.len(), available);
                buf[..to_read].copy_from_slice(&state.bytes[self.pos..self.pos + to_read]);
                self.pos += to_read;
                return Ok(to_read);
            }
            if state.done {
                return Ok(0);
            }
            if state.cancelled {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "cancelled"));
            }
            let result = self
                .buffer
                .condvar()
                .wait_timeout(state, Duration::from_millis(500))
                .unwrap();
            state = result.0;
        }
    }
}

impl std::io::Seek for StreamingReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            std::io::SeekFrom::Start(offset) => offset as i64,
            std::io::SeekFrom::Current(offset) => self.pos as i64 + offset,
            std::io::SeekFrom::End(offset) => {
                let mut state = self.buffer.state_mutex().lock().unwrap();
                loop {
                    if state.done {
                        break state.bytes.len() as i64 + offset;
                    }
                    if state.cancelled {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Interrupted,
                            "cancelled",
                        ));
                    }
                    let result = self
                        .buffer
                        .condvar()
                        .wait_timeout(state, Duration::from_millis(500))
                        .unwrap();
                    state = result.0;
                }
            }
        };
        if new_pos < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }
        let mut state = self.buffer.state_mutex().lock().unwrap();
        loop {
            if new_pos as usize <= state.bytes.len() {
                self.pos = new_pos as usize;
                return Ok(self.pos as u64);
            }
            if state.done {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "seek past end",
                ));
            }
            if state.cancelled {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "cancelled",
                ));
            }
            let result = self
                .buffer
                .condvar()
                .wait_timeout(state, Duration::from_millis(500))
                .unwrap();
            state = result.0;
        }
    }
}

/// 环形缓冲区类型别名，便于在模块间共享
pub type RingProducer = HeapProd<f32>;
pub type RingConsumer = HeapCons<f32>;

/// 创建指定容量的环形缓冲区，返回 (生产者, 消费者)
pub fn create_ring_buffer(capacity: usize) -> (RingProducer, RingConsumer) {
    let rb = HeapRb::<f32>::new(capacity);
    rb.split()
}

/// 为 StreamingReader 实现 symphonia 的 MediaSource trait
impl MediaSource for StreamingReader {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}
