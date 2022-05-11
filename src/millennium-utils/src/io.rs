// Copyright 2022 pyke.io
//           2019-2021 Tauri Programme within The Commons Conservancy
//                     [https://tauri.studio/]
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! IO helpers.

use std::io::BufRead;

/// Read a line breaking in both \n and \r.
///
/// Adapted from https://doc.rust-lang.org/std/io/trait.BufRead.html#method.read_line
pub fn read_line<R: BufRead + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> std::io::Result<usize> {
  let mut read = 0;
  loop {
    let (done, used) = {
      let available = match r.fill_buf() {
        Ok(n) => n,
        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
        Err(e) => return Err(e),
      };
      match memchr::memchr(b'\n', available) {
        Some(i) => {
          let end = i + 1;
          buf.extend_from_slice(&available[..end]);
          (true, end)
        }
        None => match memchr::memchr(b'\r', available) {
          Some(i) => {
            let end = i + 1;
            buf.extend_from_slice(&available[..end]);
            (true, end)
          }
          None => {
            buf.extend_from_slice(available);
            (false, available.len())
          }
        },
      }
    };
    r.consume(used);
    read += used;
    if done || used == 0 {
      if buf.ends_with(&[b'\n']) {
        buf.pop();
      }
      return Ok(read);
    }
  }
}
