//! A crate for [WTHOR Database](https://www.ffothello.org/informatique/la-base-wthor).

mod slice;

#[cfg(feature = "download")]
mod download;

#[cfg(feature = "download")]
pub use crate::download::{
    DownloadError,
    Downloader,
};
use {
    crate::slice::{
        as_array,
        as_array_mut,
        as_chunks,
        as_chunks_mut,
        split,
        split_mut,
    },
    heapless::Vec as HeaplessVec,
    std::{
        error::Error,
        ffi::CStr,
        fmt::{
            Display,
            Formatter,
            Result as FmtResult,
        },
        iter::zip,
    },
};

/// A jou file, which contains names of players.
#[derive(Clone, Hash, Debug)]
pub struct Jou {
    /// The centry when the file was created.
    pub created_centry: u8,
    /// The year when the file was created.
    pub created_year: u8,
    /// The month when the file was created.
    pub created_month: u8,
    /// The day when the file was created.
    pub created_day: u8,
    /// Names of players.
    pub players: Vec<HeaplessVec<u8, 19>>,
}

impl Jou {
    /// Reads a file.
    pub fn read(bytes: &[u8]) -> Result<Self, ReadError> {
        let (header, players) = split(bytes).ok_or(ReadError)?;

        let (created_centry, created_year, created_month, created_day, number_of_players) =
            read_names_header(header)?;

        Result::Ok(Self {
            created_centry,
            created_year,
            created_month,
            created_day,
            players: read_names::<19, 20>(players, number_of_players)?,
        })
    }

    /// Writes a file.
    pub fn write(&self, bytes: &mut [u8]) -> Result<(), WriteError> {
        let (header, players) = split_mut(bytes).ok_or(WriteError::InvalidInput)?;
        let number_of_players = self.players.len() as u16;

        write_names_header(
            header,
            self.created_centry,
            self.created_year,
            self.created_month,
            self.created_day,
            number_of_players,
        );

        write_names::<19, 20>(players, &self.players, number_of_players)?;
        Result::Ok(())
    }

    /// Gets the number of bytes required to write the file.
    pub fn size(&self) -> usize {
        16 + 20 * self.players.len()
    }

    /// The recommended [`stem`](std::path::Path::file_stem).
    pub const fn file_stem() -> &'static str {
        "wthor"
    }
}

/// A trn file, which contains names of tournaments.
#[derive(Clone, Hash, Debug)]
pub struct Trn {
    /// The centry when the file was created.
    pub created_centry: u8,
    /// The year when the file was created.
    pub created_year: u8,
    /// The month when the file was created.
    pub created_month: u8,
    /// The day when the file was created.
    pub created_day: u8,
    /// Names of tournaments.
    pub tournaments: Vec<HeaplessVec<u8, 25>>,
}

impl Trn {
    /// Reads a file.
    pub fn read(bytes: &[u8]) -> Result<Self, ReadError> {
        let (header, players) = split(bytes).ok_or(ReadError)?;

        let (created_centry, created_year, created_month, created_day, number_of_players) =
            read_names_header(header)?;

        Result::Ok(Self {
            created_centry,
            created_year,
            created_month,
            created_day,
            tournaments: read_names::<25, 26>(players, number_of_players)?,
        })
    }

    /// Writes a file.
    pub fn write(&self, bytes: &mut [u8]) -> Result<(), WriteError> {
        let (header, players) = split_mut(bytes).ok_or(WriteError::InvalidInput)?;
        let number_of_tournaments = self.tournaments.len() as u16;

        write_names_header(
            header,
            self.created_centry,
            self.created_year,
            self.created_month,
            self.created_day,
            number_of_tournaments,
        );

        write_names::<25, 26>(players, &self.tournaments, number_of_tournaments)?;
        Result::Ok(())
    }

    /// Gets the number of bytes required to write the file.
    pub fn size(&self) -> usize {
        16 + 26 * self.tournaments.len()
    }

    /// The recommended [`stem`](std::path::Path::file_stem).
    pub const fn file_stem() -> &'static str {
        "wthor"
    }
}

/// A wtb file, which contains `8x8` Othello games.
#[derive(Clone, Hash, Debug)]
pub struct Wtb {
    /// The centry when the file was created.
    pub created_centry: u8,
    /// The year when the file was created.
    pub created_year: u8,
    /// The month when the file was created.
    pub created_month: u8,
    /// The day when the file was created.
    pub created_day: u8,
    /// The year when the games was played.
    pub year: u16,
    /// A number used to calculate [`Game::theoretical_score`].
    /// The value `0` is equivalent to the value `22` in files after 01/01/2001.
    pub calculation_depth: u8,
    /// Othello games.
    pub games: Vec<Game<60>>,
}

impl Wtb {
    /// Reads a file.
    pub fn read(bytes: &[u8]) -> Result<Self, ReadError> {
        let (header, games) = split(bytes).ok_or(ReadError)?;

        let (
            created_centry,
            created_year,
            created_month,
            created_day,
            number_of_games,
            year,
            size_of_board,
            calculation_depth,
        ) = read_games_header(header)?;

        if size_of_board != 0 && size_of_board != 8 {
            return Result::Err(ReadError);
        }

        Result::Ok(Self {
            created_centry,
            created_year,
            created_month,
            created_day,
            year,
            calculation_depth,
            games: read_games::<60, 68>(games, number_of_games)?,
        })
    }

    /// Writes a file.
    pub fn write(&self, bytes: &mut [u8]) -> Result<(), WriteError> {
        let (header, games) = split_mut(bytes).ok_or(WriteError::InvalidInput)?;
        let number_of_games = self.games.len() as u32;

        write_games_header(
            header,
            self.created_centry,
            self.created_year,
            self.created_month,
            self.created_day,
            number_of_games,
            self.year,
            8,
            self.calculation_depth,
        );

        write_games::<60, 68>(games, &self.games, number_of_games)?;
        Result::Ok(())
    }

    /// Gets the number of bytes required to write the file.
    pub fn size(&self) -> usize {
        16 + 68 * self.games.len()
    }

    /// The recommended [`stem`](std::path::Path::file_stem).
    pub fn file_stem(year: u16) -> String {
        format!("wth_{year}")
    }
}

/// A wtd file, which contains `10x10` Othello games.
#[derive(Clone, Hash, Debug)]
pub struct Wtd {
    /// The centry when the file was created.
    pub created_centry: u8,
    /// The year when the file was created.
    pub created_year: u8,
    /// The month when the file was created.
    pub created_month: u8,
    /// The day when the file was created.
    pub created_day: u8,
    /// The year when the games was played.
    pub year: u16,
    /// A number used to calculate [`Game::theoretical_score`].
    /// The value `0` is equivalent to `22` in files after 01/01/2001.
    pub calculation_depth: u8,
    /// Othello games.
    pub games: Vec<Game<96>>,
}

impl Wtd {
    /// Reads a file.
    pub fn read(bytes: &[u8]) -> Result<Self, ReadError> {
        let (header, games) = split(bytes).ok_or(ReadError)?;

        let (
            created_centry,
            created_year,
            created_month,
            created_day,
            number_of_games,
            year,
            size_of_board,
            calculation_depth,
        ) = read_games_header(header)?;

        if size_of_board != 10 {
            return Result::Err(ReadError);
        }

        Result::Ok(Self {
            created_centry,
            created_year,
            created_month,
            created_day,
            year,
            calculation_depth,
            games: read_games::<96, 104>(games, number_of_games)?,
        })
    }

    /// Writes a file.
    pub fn write(&self, bytes: &mut [u8]) -> Result<(), WriteError> {
        let (header, games) = split_mut(bytes).ok_or(WriteError::InvalidInput)?;
        let number_of_games = self.games.len() as u32;

        write_games_header(
            header,
            self.created_centry,
            self.created_year,
            self.created_month,
            self.created_day,
            number_of_games,
            self.year,
            10,
            self.calculation_depth,
        );

        write_games::<96, 104>(games, &self.games, number_of_games)?;
        Result::Ok(())
    }

    /// Gets the number of bytes required to write the file.
    pub fn size(&self) -> usize {
        16 + 104 * self.games.len()
    }

    /// The recommended [`stem`](std::path::Path::file_stem).
    pub fn file_stem(year: u16) -> String {
        format!("wth_{year}")
    }
}

/// A Othello game.
#[derive(Clone, Hash, Debug)]
pub struct Game<const N: usize> {
    /// The index of the tournament.
    pub tournament: u16,
    /// The index of the black player.
    pub black_player: u16,
    /// The index of the white player.
    pub white_player: u16,
    /// The final number of black disks.
    pub score: u8,
    /// The number of black disks if the black player had made the best moves since a move when the number of empty squares is equal to `calculation_depth`.
    pub theoretical_score: u8,
    /// The moves.
    pub moves: [u8; N],
}

fn read_header(bytes: &[u8; 16]) -> (u8, u8, u8, u8, u32, u16, u16, u8, u8, u8) {
    (
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        u32::from_le_bytes(*as_array(&bytes[4..=7]).unwrap()),
        u16::from_le_bytes(*as_array(&bytes[8..=9]).unwrap()),
        u16::from_le_bytes(*as_array(&bytes[10..=11]).unwrap()),
        bytes[12],
        bytes[13],
        bytes[14],
    )
}

#[allow(clippy::too_many_arguments)]
fn write_header(
    bytes: &mut [u8; 16],
    created_centry: u8,
    created_year: u8,
    created_month: u8,
    created_day: u8,
    n1: u32,
    n2: u16,
    game_year: u16,
    p1: u8,
    p2: u8,
    p3: u8,
) {
    bytes[0] = created_centry;
    bytes[1] = created_year;
    bytes[2] = created_month;
    bytes[3] = created_day;
    *as_array_mut(&mut bytes[4..=7]).unwrap() = n1.to_le_bytes();
    *as_array_mut(&mut bytes[8..=9]).unwrap() = n2.to_le_bytes();
    *as_array_mut(&mut bytes[10..=11]).unwrap() = game_year.to_le_bytes();
    bytes[12] = p1;
    bytes[13] = p2;
    bytes[14] = p3;
}

fn read_names_header(bytes: &[u8; 16]) -> Result<(u8, u8, u8, u8, u16), ReadError> {
    let (
        created_centry,
        created_year,
        created_month,
        created_day,
        n1,
        number_of_names,
        game_year,
        p1,
        p2,
        _,
    ) = read_header(bytes);

    if n1 != 0 || game_year != 0 || p1 != 0 || p2 != 0 {
        return Result::Err(ReadError);
    }

    Result::Ok((
        created_centry,
        created_year,
        created_month,
        created_day,
        number_of_names,
    ))
}

fn write_names_header(
    bytes: &mut [u8; 16],
    created_centry: u8,
    created_year: u8,
    created_month: u8,
    created_day: u8,
    number_of_names: u16,
) {
    write_header(
        bytes,
        created_centry,
        created_year,
        created_month,
        created_day,
        0,
        number_of_names,
        0,
        0,
        0,
        0,
    );
}

#[allow(clippy::type_complexity)]
fn read_games_header(bytes: &[u8; 16]) -> Result<(u8, u8, u8, u8, u32, u16, u8, u8), ReadError> {
    let (
        created_centry,
        created_year,
        created_month,
        created_day,
        number_of_games,
        n2,
        year,
        size_of_board,
        game_type,
        calculation_depth,
    ) = read_header(bytes);

    if n2 != 0 || game_type != 0 {
        return Result::Err(ReadError);
    }

    Result::Ok((
        created_centry,
        created_year,
        created_month,
        created_day,
        number_of_games,
        year,
        size_of_board,
        calculation_depth,
    ))
}

#[allow(clippy::too_many_arguments)]
fn write_games_header(
    bytes: &mut [u8; 16],
    created_centry: u8,
    created_year: u8,
    created_month: u8,
    created_day: u8,
    number_of_games: u32,
    year: u16,
    size_of_board: u8,
    calculation_depth: u8,
) {
    write_header(
        bytes,
        created_centry,
        created_year,
        created_month,
        created_day,
        number_of_games,
        0,
        year,
        size_of_board,
        0,
        calculation_depth,
    );
}

fn read_names<const N: usize, const SN: usize>(
    bytes: &[u8],
    count: u16,
) -> Result<Vec<HeaplessVec<u8, N>>, ReadError> {
    let count = count as usize;
    let chunks = as_chunks::<_, SN>(bytes, count).ok_or(ReadError)?;
    let mut result = Vec::with_capacity(count);

    for chunk in chunks {
        result.push(
            HeaplessVec::from_slice(
                CStr::from_bytes_until_nul(chunk)
                    .map_err(|_| ReadError)?
                    .to_bytes(),
            )
            .map_err(|_| ReadError)?,
        );
    }

    Result::Ok(result)
}

fn write_names<const N: usize, const SN: usize>(
    bytes: &mut [u8],
    names: &[HeaplessVec<u8, N>],
    count: u16,
) -> Result<(), WriteError> {
    if names.len() != count as usize {
        return Result::Err(WriteError::TooManyElements);
    }

    let chunks = as_chunks_mut::<_, SN>(bytes, names.len()).ok_or(WriteError::InvalidInput)?;

    if chunks.len() != names.len() {
        return Result::Err(WriteError::InvalidInput);
    }

    for (chunk, name) in zip(chunks, names) {
        chunk[0..name.len()].copy_from_slice(name);
        chunk[name.len()] = b'0';
    }

    Result::Ok(())
}

fn read_games<const N: usize, const S: usize>(
    bytes: &[u8],
    count: u32,
) -> Result<Vec<Game<N>>, ReadError> {
    let count = count as usize;
    let chunks = as_chunks::<_, S>(bytes, count).ok_or(ReadError)?;
    let mut result = Vec::with_capacity(count);

    for chunk in chunks {
        result.push(Game {
            tournament: u16::from_le_bytes(*as_array(&chunk[0..=1]).unwrap()),
            black_player: u16::from_le_bytes(*as_array(&chunk[2..=3]).unwrap()),
            white_player: u16::from_le_bytes(*as_array(&chunk[4..=5]).unwrap()),
            score: chunk[6],
            theoretical_score: chunk[7],
            moves: *as_array(&chunk[8..]).ok_or(ReadError)?,
        });
    }

    Result::Ok(result)
}

fn write_games<const N: usize, const S: usize>(
    bytes: &mut [u8],
    games: &[Game<N>],
    count: u32,
) -> Result<(), WriteError> {
    if games.len() != count as usize {
        return Result::Err(WriteError::TooManyElements);
    }

    let chunks = as_chunks_mut::<_, S>(bytes, games.len()).ok_or(WriteError::InvalidInput)?;

    if chunks.len() != games.len() {
        return Result::Err(WriteError::InvalidInput);
    }

    for (chunk, game) in zip(chunks, games) {
        *as_array_mut(&mut chunk[0..=1]).unwrap() = game.tournament.to_le_bytes();
        *as_array_mut(&mut chunk[2..=3]).unwrap() = game.black_player.to_le_bytes();
        *as_array_mut(&mut chunk[4..=5]).unwrap() = game.white_player.to_le_bytes();
        chunk[6] = game.score;
        chunk[7] = game.theoretical_score;
        *as_array_mut(&mut chunk[8..]).ok_or(WriteError::InvalidInput)? = game.moves;
    }

    Result::Ok(())
}

/// An error while reading.
/// This indicates that the input is an invalid file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ReadError;

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("the input is invalid")
    }
}

impl Error for ReadError {}

/// An error while writing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WriteError {
    /// The length of the input slice is not equals to the file size.
    InvalidInput,
    /// The input has too many elements.
    TooManyElements,
}

impl Display for WriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self {
            Self::InvalidInput => "the input is invalid",
            Self::TooManyElements => "the elements is too many",
        })
    }
}

impl Error for WriteError {}
