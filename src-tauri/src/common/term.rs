pub mod terminal {
    use std::io::{Read, Write};

    use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
    use serde::{Deserialize, Serialize};
    use snafu::Snafu;
    use uuid::Uuid;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(remote = "PtySize")]
    pub struct PtySizeDef {
        pub rows: u16,
        pub cols: u16,
        pub pixel_width: u16,
        pub pixel_height: u16
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct TermInfo {
        pub id: Uuid,
        pub title: Option<String>,
        pub command: String,
        pub args: Option<Vec<String>>,

        #[serde(with = "PtySizeDef")]
        pub size: PtySize
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Snafu)]
    pub enum TerminalError {
        #[snafu(display("Failed to open PTY"))]
        Open,

        #[snafu(display("Failed to spawn command"))]
        CommandSpawn,

        #[snafu(display("Failed to attach reader"))]
        ReaderAttach,

        #[snafu(display("Failed to attach writer"))]
        WriterAttach,

        #[snafu(display("Failed to resize window"))]
        ResizeWindow,

        #[snafu(display("Failed to read from PTY"))]
        Read,

        #[snafu(display("Failed to write to PTY"))]
        Write,

        #[snafu(display("Failed to execute child operation {op}"))]
        ChildOperation {op: String},

        #[snafu(display("PTY already closed"))]
        ClosedPty
    }

    pub struct Terminal {
        pub info: TermInfo,
        pub ptys: PtyPair,
        pub child: Box<dyn Child + Send + Sync>
    }

    impl Terminal {
        pub fn new(rows: u16, cols: u16, command: String, title: Option<String>, args: Option<Vec<String>>) -> Result<Self, TerminalError> {
            let info = TermInfo {
                id: Uuid::new_v4(),
                title,
                command: command.clone(),
                args: args.clone(),
                size: PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }
            };

            let pty = native_pty_system().openpty(info.size).map_err(|_| TerminalError::Open)?;
            let mut cmd = CommandBuilder::new(command);
            if let Some(a) = args {
                cmd.args(a);
            }
            let child = pty.slave.spawn_command(cmd).map_err(|_| TerminalError::CommandSpawn)?;
            let reader = pty.master.try_clone_reader().map_err(|_| TerminalError::ReaderAttach)?;
            let writer = pty.master.take_writer().map_err(|_| TerminalError::WriterAttach)?;

            Ok(Terminal {
                info,
                ptys: pty,
                child
            })
        }

        pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), TerminalError> {
            if self.status().is_some() {
                return Err(TerminalError::ClosedPty);
            }
            self.ptys.master.resize(PtySize {
                rows,
                cols,
                pixel_height: 0,
                pixel_width: 0
            }).map_err(|_| TerminalError::ResizeWindow)
        }

        pub fn id(&self) -> Uuid {
            self.info.id
        }

        /*pub fn read(&mut self) -> Result<String, TerminalError> {
            if self.status().is_some() {
                return Err(TerminalError::ClosedPty);
            }
            let mut buf = String::new();
            self.reader.read_to_string(&mut buf).map_err(|_| TerminalError::Read)?;
            Ok(buf)
        }

        pub fn write(&mut self, content: String) -> Result<(), TerminalError> {
            if self.status().is_some() {
                return Err(TerminalError::ClosedPty);
            }
            self.writer.write_all(content.as_bytes()).map_err(|_| TerminalError::Write)?;
            Ok(())
        }*/

        pub fn kill(&mut self) -> Result<(), TerminalError> {
            if self.status().is_some() {
                return Err(TerminalError::ClosedPty);
            }
            self.child.kill().map_err(|_| TerminalError::ChildOperation { op: "Kill process".to_string() })?;
            Ok(())
        }

        pub fn status(&mut self) -> Option<u32> {
            match self.child.try_wait() {
                Ok(s) => match s {
                    Some(e) => Some(e.exit_code()),
                    None => None
                },
                Err(_) => None
            }
        }
    }
}