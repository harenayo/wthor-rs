//! A crate for [WTHOR Database](https://www.ffothello.org/informatique/la-base-wthor).

#[cfg(feature = "download")]
mod download;

#[cfg(feature = "download")]
pub use crate::download::*;
use {
    heapless::Vec as HeaplessVec,
    othello::Position,
    std::{
        error::Error,
        fmt::{
            Display,
            Formatter,
            Result as FmtResult,
        },
        io::{
            Error as IoError,
            Read,
            Write,
        },
        iter::repeat,
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
    pub fn read(mut r: impl Read) -> Result<Self, ReadError> {
        let (created_centry, created_year, created_month, created_day, number_of_players) =
            read_names_header(&mut r)?;

        Result::Ok(Self {
            created_centry,
            created_year,
            created_month,
            created_day,
            players: read_names(&mut r, number_of_players)?,
        })
    }

    /// Writes a file.
    pub fn write(&self, mut w: impl Write) -> Result<(), WriteError> {
        let number_of_players = self.players.len() as u16;

        write_names_header(
            &mut w,
            self.created_centry,
            self.created_year,
            self.created_month,
            self.created_day,
            number_of_players,
        )?;

        write_names(&mut w, &self.players, number_of_players)?;
        Result::Ok(())
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
    pub fn read(mut r: impl Read) -> Result<Self, ReadError> {
        let (created_centry, created_year, created_month, created_day, number_of_players) =
            read_names_header(&mut r)?;

        Result::Ok(Self {
            created_centry,
            created_year,
            created_month,
            created_day,
            tournaments: read_names(&mut r, number_of_players)?,
        })
    }

    /// Writes a file.
    pub fn write(&self, mut w: impl Write) -> Result<(), WriteError> {
        let number_of_tournaments = self.tournaments.len() as u16;

        write_names_header(
            &mut w,
            self.created_centry,
            self.created_year,
            self.created_month,
            self.created_day,
            number_of_tournaments,
        )?;

        write_names(&mut w, &self.tournaments, number_of_tournaments)?;
        Result::Ok(())
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
    pub games: Vec<GameInfo>,
}

impl Wtb {
    /// Reads a file.
    pub fn read(mut r: impl Read) -> Result<Self, ReadError> {
        let (
            created_centry,
            created_year,
            created_month,
            created_day,
            number_of_games,
            year,
            size_of_board,
            calculation_depth,
        ) = read_games_header(&mut r)?;

        if size_of_board != 0 && size_of_board != 8 {
            return Result::Err(ReadError::InvalidFormat);
        }

        Result::Ok(Self {
            created_centry,
            created_year,
            created_month,
            created_day,
            year,
            calculation_depth,
            games: read_games(&mut r, number_of_games)?,
        })
    }

    /// Writes a file.
    pub fn write(&self, mut w: impl Write) -> Result<(), WriteError> {
        let number_of_games = self.games.len() as u32;

        write_games_header(
            &mut w,
            self.created_centry,
            self.created_year,
            self.created_month,
            self.created_day,
            number_of_games,
            self.year,
            8,
            self.calculation_depth,
        )?;

        write_games(&mut w, &self.games, number_of_games)?;
        Result::Ok(())
    }
}

/// A Othello game.
#[derive(Clone, Hash, Debug)]
pub struct GameInfo {
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
    pub moves: HeaplessVec<Position, 60>,
}

fn read<const N: usize>(r: &mut impl Read) -> Result<[u8; N], ReadError> {
    let mut result = [0; N];
    r.read_exact(&mut result)?;
    Result::Ok(result)
}

#[allow(clippy::type_complexity)]
fn read_header(
    r: &mut impl Read,
) -> Result<(u8, u8, u8, u8, u32, u16, u16, u8, u8, u8), ReadError> {
    let result = (
        read::<1>(r)?[0],
        read::<1>(r)?[0],
        read::<1>(r)?[0],
        read::<1>(r)?[0],
        u32::from_le_bytes(read(r)?),
        u16::from_le_bytes(read(r)?),
        u16::from_le_bytes(read(r)?),
        read::<1>(r)?[0],
        read::<1>(r)?[0],
        read::<1>(r)?[0],
    );

    read::<1>(r)?;
    Result::Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn write_header(
    w: &mut impl Write,
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
) -> Result<(), WriteError> {
    w.write_all(&[created_centry, created_year, created_month, created_day])?;
    w.write_all(&n1.to_le_bytes())?;
    w.write_all(&n2.to_le_bytes())?;
    w.write_all(&game_year.to_le_bytes())?;
    w.write_all(&[p1, p2, p3, 0])?;
    Result::Ok(())
}

fn read_names_header(r: &mut impl Read) -> Result<(u8, u8, u8, u8, u16), ReadError> {
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
    ) = read_header(r)?;

    if n1 != 0 || game_year != 0 || p1 != 0 || p2 != 0 {
        return Result::Err(ReadError::InvalidFormat);
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
    w: &mut impl Write,
    created_centry: u8,
    created_year: u8,
    created_month: u8,
    created_day: u8,
    number_of_names: u16,
) -> Result<(), WriteError> {
    write_header(
        w,
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
    )
}

#[allow(clippy::type_complexity)]
fn read_games_header(r: &mut impl Read) -> Result<(u8, u8, u8, u8, u32, u16, u8, u8), ReadError> {
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
    ) = read_header(r)?;

    if n2 != 0 || game_type != 0 {
        return Result::Err(ReadError::InvalidFormat);
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
    w: &mut impl Write,
    created_centry: u8,
    created_year: u8,
    created_month: u8,
    created_day: u8,
    number_of_games: u32,
    year: u16,
    size_of_board: u8,
    calculation_depth: u8,
) -> Result<(), WriteError> {
    write_header(
        w,
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
    )
}

fn read_names<const N: usize>(
    r: &mut impl Read,
    count: u16,
) -> Result<Vec<HeaplessVec<u8, N>>, ReadError> {
    let result = (0..count)
        .map(|_| {
            let result = read::<N>(r)?
                .into_iter()
                .take_while(|c| *c != b'0')
                .collect();

            read::<1>(r)?;
            Result::Ok(result)
        })
        .collect();

    if read::<1>(r).is_ok() {
        return Result::Err(ReadError::InvalidFormat);
    }

    result
}

fn write_names<const N: usize>(
    w: &mut impl Write,
    names: &[HeaplessVec<u8, N>],
    count: u16,
) -> Result<(), WriteError> {
    if names.len() != count as usize {
        return Result::Err(WriteError::TooManyElements);
    }

    for name in names {
        let mut name = name.clone();
        name.extend(repeat(b'0').take(N - name.len()));
        w.write_all(&name)?;
        w.write_all(&[b'0'])?;
    }

    Result::Ok(())
}

fn read_games(r: &mut impl Read, count: u32) -> Result<Vec<GameInfo>, ReadError> {
    let result = (0..count)
        .map(|_| {
            Result::Ok(GameInfo {
                tournament: u16::from_le_bytes(read(r)?),
                black_player: u16::from_le_bytes(read(r)?),
                white_player: u16::from_le_bytes(read(r)?),
                score: read::<1>(r)?[0],
                theoretical_score: read::<1>(r)?[0],
                moves: {
                    let moves: HeaplessVec<_, 60> = read::<60>(r)?
                        .into_iter()
                        .take_while(|r#move| *r#move != 0)
                        .map(|r#move| Position::at(r#move / 10 - 1, r#move % 10 - 1))
                        .collect::<Option<_>>()
                        .ok_or(ReadError::InvalidFormat)?;

                    if let Option::Some(r#move) = moves.iter().next() {
                        if *r#move != Position::at(4, 5).unwrap() {
                            return Result::Err(ReadError::InvalidFormat);
                        }
                    }

                    moves
                },
            })
        })
        .collect();

    if read::<1>(r).is_ok() {
        return Result::Err(ReadError::InvalidFormat);
    }

    result
}

fn write_games(w: &mut impl Write, games: &[GameInfo], count: u32) -> Result<(), WriteError> {
    if games.len() != count as usize {
        return Result::Err(WriteError::TooManyElements);
    }

    for game in games {
        w.write_all(&game.tournament.to_le_bytes())?;
        w.write_all(&game.black_player.to_le_bytes())?;
        w.write_all(&game.white_player.to_le_bytes())?;
        w.write_all(&[game.score, game.theoretical_score])?;

        w.write_all(
            &game
                .moves
                .iter()
                .map(|r#move| 10 * r#move.row() + r#move.column() + 11)
                .chain(repeat(0))
                .take(60)
                .collect::<HeaplessVec<_, 60>>(),
        )?;
    }

    Result::Ok(())
}

/// An error while reading.
#[derive(Debug)]
pub enum ReadError {
    /// The input is an invalid file.
    InvalidFormat,
    /// See [`Error`](IoError).
    Io(IoError),
}

impl Display for ReadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::InvalidFormat => formatter.write_str("the input is invalid"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl Error for ReadError {}

impl From<IoError> for ReadError {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}

/// An error while writing.
#[derive(Debug)]
pub enum WriteError {
    /// The file has too many elements.
    TooManyElements,
    /// See [`Error`](IoError).
    Io(IoError),
}

impl Display for WriteError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::TooManyElements => formatter.write_str("the elements is too many"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl Error for WriteError {}

impl From<IoError> for WriteError {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}
