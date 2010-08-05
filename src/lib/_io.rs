import std.os;
import std._str;
import std._vec;

type buf_reader = unsafe obj {
  fn read() -> vec[u8];
};

type buf_writer = unsafe obj {
  fn write(vec[u8] v);
};

fn default_bufsz() -> uint {
  ret 4096u;
}

fn new_buf() -> vec[u8] {
  ret _vec.alloc[u8](default_bufsz());
}

fn new_buf_reader(str path) -> buf_reader {

  unsafe obj fd_buf_reader(int fd, mutable vec[u8] buf) {

    fn read() -> vec[u8] {

      // Ensure our buf is singly-referenced.
      if (_vec.rustrt.refcount[u8](buf) != 1u) {
        buf = new_buf();
      }

      auto len = _vec.len[u8](buf);
      auto vbuf = _vec.buf[u8](buf);
      auto count = os.libc.read(fd, vbuf, len);

      if (count < 0) {
        log "error filling buffer";
        log sys.rustrt.last_os_error();
        fail;
      } else {
        ret buf;
      }
    }

    drop {
      os.libc.close(fd);
    }
  }

  auto fd = os.libc.open(_str.buf(path),
                         os.libc_constants.O_RDONLY() |
                         os.libc_constants.O_BINARY(),
                         0u);

  if (fd < 0) {
    log "error opening file for reading";
    log sys.rustrt.last_os_error();
    fail;
  }
  ret fd_buf_reader(fd, new_buf());
}

type fileflag = tag(append(), create(), truncate());

fn new_buf_writer(str path, vec[fileflag] flags) -> buf_writer {

  unsafe obj fd_buf_writer(int fd) {

    fn write(vec[u8] v) {
      auto len = _vec.len[u8](v);
      auto count = 0u;
      auto vbuf;
      while (count < len) {
        vbuf = _vec.buf_off[u8](v, count);
        auto nout = os.libc.write(fd, vbuf, len);
        if (nout < 0) {
          log "error dumping buffer";
          log sys.rustrt.last_os_error();
          fail;
        }
        count += nout as uint;
      }
    }

    drop {
      os.libc.close(fd);
    }
  }

  let int fflags =
    os.libc_constants.O_WRONLY() |
    os.libc_constants.O_BINARY();

  for (fileflag f in flags) {
    alt (f) {
      case (append())   { fflags |= os.libc_constants.O_APPEND(); }
      case (create())   { fflags |= os.libc_constants.O_CREAT(); }
      case (truncate()) { fflags |= os.libc_constants.O_TRUNC(); }
    }
  }

  auto fd = os.libc.open(_str.buf(path),
                         fflags,
                         os.libc_constants.S_IRUSR() |
                         os.libc_constants.S_IWUSR());

  if (fd < 0) {
    log "error opening file for writing";
    log sys.rustrt.last_os_error();
    fail;
  }
  ret fd_buf_writer(fd);
}

type formatter[T] = fn(&T x) -> vec[u8];

type writer[T] = unsafe obj { fn write(&T x); };

fn mk_writer[T](str path,
                vec[fileflag] flags,
                &formatter[T] fmt)
  -> writer[T]
{
  unsafe obj w[T](buf_writer out, formatter[T] fmt) {
    fn write(&T x) {
      out.write(fmt(x));
    }
  }
  ret w[T](new_buf_writer(path, flags), fmt);
}

/* TODO: int_writer, uint_writer, ... */

fn str_writer(str path, vec[fileflag] flags) -> writer[str] {
  auto fmt = _str.bytes; // FIXME (issue #90)
  ret mk_writer[str](path, flags, fmt);
}

fn vec_writer[T](str path,
                 vec[fileflag] flags,
                 &formatter[T] inner)
  -> writer[vec[T]]
{
  fn fmt[T](&vec[T] v, &formatter[T] inner) -> vec[u8] {
    let vec[u8] res = _str.bytes("vec(");
    auto first = true;
    for (T x in v) {
      if (!first) {
        res += _str.bytes(", ");
      } else {
        first = false;
      }
      res += inner(x);
    }
    res += _str.bytes(")\n");
    ret res;
  }

  ret mk_writer[vec[T]](path, flags, bind fmt[T](_, inner));
}
