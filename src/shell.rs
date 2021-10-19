use crate::*;
use core::fmt::Write;
use rtic::Mutex;
use ushell::*;

pub const CMD_MAX_LEN: usize = 32;
pub const AUTOCOMPLETE_LEN: usize = 4;
pub const AUTOCOMPLETE: [&str; AUTOCOMPLETE_LEN] = ["clear", "blink ", "help", "help pinout"];

pub type Uart = serial::Serial<stm32::USART2, serial::BasicConfig>;
pub type Autocomplete = autocomplete::StaticAutocomplete<{ AUTOCOMPLETE_LEN }>;
pub type History = LRUHistory<{ CMD_MAX_LEN }, 32>;
pub type Shell = UShell<Uart, Autocomplete, History, { CMD_MAX_LEN }>;
pub type Env<'a> = app::uart_rx::SharedResources<'a>;
pub type SpinError = ushell::SpinError<Uart, ()>;

impl Environment<Uart, Autocomplete, History, (), { CMD_MAX_LEN }> for Env<'_> {
    fn command(&mut self, shell: &mut Shell, cmd: &str, args: &str) -> Result<(), SpinError> {
        match cmd {
            "help" => self.help_cmd(shell, args),
            "blink" => self.blink_cmd(shell, args),
            "clear" => shell.clear(),
            "" => shell.write_str("\r\n").map_err(ShellError::FormatError),
            _ => write!(shell, "\r\nunknown command: \"{}\"\r\n", cmd)
                .map_err(ShellError::FormatError),
        }
        .and_then(|_| shell.write_str("» ").map_err(ShellError::FormatError))
        .map_err(SpinError::ShellError)
    }

    fn control(&mut self, _shell: &mut Shell, _code: u8) -> Result<(), SpinError> {
        Ok(())
    }
}

impl Env<'_> {
    fn help_cmd(&mut self, shell: &mut Shell, args: &str) -> ShellResult<Uart> {
        match args {
            "pinout" => shell.write_str(
                "\r\n\
                \x20           STM32G0xxFx  \r\n\
                \x20          ╔═══════════╗ \r\n\
                \x20  PB7|PB8 ╣1 ¤      20╠ PB3|PB4|PB5|PB6\r\n\
                \x20 PC9|PC14 ╣2        19╠ PA14|PA15      \r\n\
                \x20(LED)PC15 ╣3        18╠ PA13           \r\n\
                \x20      Vdd ╣4        17╠ PA12[PA10]     \r\n\
                \x20      Vss ╣5        16╠ PA11[PA9]      \r\n\
                \x20     nRst ╣6        15╠ PA8|PB0|PB1|PB2\r\n\
                \x20      PA0 ╣7        14╠ PA7            \r\n\
                \x20      PA1 ╣8        13╠ PA6            \r\n\
                \x20(TX)  PA2 ╣9        12╠ PA5            \r\n\
                \x20(RX)  PA3 ╣10       11╠ PA4            \r\n\
                \x20          ╚═══════════╝ \r\n\r\n",
            ),
            _ => shell.write_str(
                "\r\n\
                Placebo Shell v0.0.1\r\n\r\n\
                COMMANDS:\r\n\
                \x20 blink <freq>     Set blink freqency\r\n\
                \x20 help [pinout]    Print help message\r\n\
                \x20 clear            Clear screen\r\n\r\n",
            ),
        }
        .map_err(ShellError::FormatError)
    }

    fn blink_cmd(&mut self, shell: &mut Shell, args: &str) -> ShellResult<Uart> {
        match btoi::btoi::<u32>(args.as_bytes()) {
            Ok(freq) if freq <= 1_000_000 => {
                self.timer.lock(|timer| timer.start((freq * 2).hz()));
                shell.write_str("\r\n")
            }
            _ => write!(shell, "\r\nunsupported blink frequency: \"{}\"\r\n", args),
        }
        .map_err(ShellError::FormatError)
    }
}