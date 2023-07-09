# NEXTSPEAKER

NEXTSPEAKER is a speaker chooser,
selecting a speaker from a list of participants.

## Goals

* Be unpredictable
* Avoid choosing too-recently chosen speakers
* Avoid neglecting to choose participants for too long
* Avoid aggressively focusing on newbies to the participant group

## Building

To compile and use this [Rust](https://www.rust-lang.org/) software,
you can install [rustup](https://www.rust-lang.org/tools/install)
and use `cargo` as desired,
e.g., as shown below.

## Usage

Create two text files.

* A list of participants, one name per line, without duplicates
* A list of previous selections, one name per line

## Example

There are two text files.

    bash$ wc -l *.txt
    9 history.txt
    21 participants.txt
    30 total
    bash$ sed 3q participants.txt 
    Alice
    Abram
    Adam
    bash$ sed 3q history.txt 
    Edith
    Earnie
    Estelle

We review the usage.

    bash$ cargo run -- --help
        Finished dev [unoptimized + debuginfo] target(s) in 0.03s
         Running `target/debug/nextspeaker --help`
    Usage: nextspeaker [OPTIONS] <PARTICIPANTS>

    Arguments:
      <PARTICIPANTS>

    Options:
          --history <HISTORY>
          --history-halflife <HISTORY_HALFLIFE>  [default: 10]
          --verbosity <VERBOSITY>
      -h, --help                                 Print help information
    bash$

The next speaker is selected.

    bash$ cargo run -- participants.txt --history history.txt
        Finished dev [unoptimized + debuginfo] target(s) in 0.02s
        Running `target/debug/nextspeaker participants.txt --history history.txt`
    Alice
    bash$ 

We record the selection in the history.

    bash$ echo Alice >> history.txt

## History

The core selection algorithm here is based
on [the memoradical flashcards app](https://github.com/ecashin/memoradical).

Balancing the entire-history per-participant selection rate
might not always be ideal.
New participants would not *always* be selected,
but they'd be favored heavily.
To avoid overwhelming newcomers to the group,
a "half life" is used to make the historical effect
of participation decay over time.
The default half life is ten selections.

## Simulation

A statistician would say that this program is fair "in expectation."
You might *happen to* get results that seem unfair toward a participant
who does or doesn't want to get chosen.
To help provide a feel for the kind of results you can expect,
there's a simulation mode, where you specify how many times the program
is used in the simulation.

The output shows each participant with a count
of the number of times that participant was selected.
By running the program 1000 times and seeing what would have happened,
one can get a feel for what the selected settings are doing.

Here is an example.

    bash$ cat participants.txt
    Alice
    Abram
    Adam
    Barbara
    Ben
    Bob
    Carl
    Cassandra
    Charli
    David
    Debora
    Doug
    Edith
    Earnie
    Estelle
    Francis
    Franklin
    Frederick
    Gladys
    Gabriel
    Gayle
    bash$

Note from the history that Earnie has had many turns,
but they were concentrated in the past.

    bash$ cat history.txt
    Earnie
    Earnie
    Earnie
    Earnie
    Earnie
    Earnie
    Earnie
    Earnie
    Earnie
    Earnie
    Edith
    Earnie
    Estelle
    Earnie
    Earnie
    Francis
    Franklin
    Frederick
    Gladys
    Gabriel
    Gayle
    bash$

With the default half-life of ten events, Earnie is selected
in the 1000 simulated runs 1.6% of the time.

    bash$ cargo run -- --history history.txt --n-simulations 1000 participants.txt
        Finished dev [unoptimized + debuginfo] target(s) in 0.04s
         Running `target/debug/nextspeaker --history history.txt --n-simulations 1000 participants.txt`
         Alice: 64
         Abram: 86
          Adam: 75
       Barbara: 67
           Ben: 46
           Bob: 66
          Carl: 60
     Cassandra: 64
        Charli: 68
         David: 67
        Debora: 59
          Doug: 65
         Edith: 55
        Earnie: 16
       Estelle: 39
       Francis: 45
      Franklin: 58
     Frederick: 0
        Gladys: 0
       Gabriel: 0
         Gayle: 0
    bash$

If that seems unfair, you can cause the effect of history to last longer.
Increasing the half life to 100 below makes Earnie less likely to be selected.
In this second simulation, Earnie is chosen 0.9% of the time, down from 1.6%
when using the default half life.

    bash$ cargo run -- --history history.txt --n-simulations 1000 --history-halflife 100 participants.txt
        Finished dev [unoptimized + debuginfo] target(s) in 0.05s
         Running `target/debug/nextspeaker --history history.txt --n-simulations 1000 --history-halflife 100 participants.txt`
         Alice: 64
         Abram: 69
          Adam: 75
       Barbara: 66
           Ben: 55
           Bob: 74
          Carl: 54
     Cassandra: 72
        Charli: 67
         David: 74
        Debora: 67
          Doug: 72
         Edith: 56
        Earnie: 9
       Estelle: 39
       Francis: 40
      Franklin: 47
     Frederick: 0
        Gladys: 0
       Gabriel: 0
         Gayle: 0
    bash$

