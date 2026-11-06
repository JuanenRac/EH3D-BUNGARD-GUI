#!/usr/bin/env python3
"""Makes the spoken notice between the stages of a job (voices/LANGUAGE/GENDER/stage.wav).

The notice says that the drilling is done and the routing comes now. It is made like the other notices of this folder, with the neural
text-to-speech program Piper (https://github.com/OHF-Voice/piper1-gpl), one voice after the other, so the computer is not loaded.

What it needs and does not bring: Piper installed for the Python that runs this script (`python -m piper --help` must work) and the models
(.onnx with its .json) of the voices listed in SOURCES.txt, in one folder, named as the voice is (for example `es_ES-davefx-medium.onnx`).
It downloads nothing and installs nothing. The voice of each LANGUAGE/GENDER (and its speaker, when the model has several) is read from
SOURCES.txt, so a voice that is changed there is changed here.

    python voices/make_stage.py --models PATH_TO_THE_MODELS            all the voices that have no stage.wav yet
    python voices/make_stage.py --models PATH_TO_THE_MODELS --only es/f
    python voices/make_stage.py --models PATH_TO_THE_MODELS --force    makes them again
    python voices/make_stage.py --list                                 shows what would be done and what is missing, without Piper

The sentences are in the table below; those of the languages other than the first eight wait for a review by a native speaker.
"""

import argparse
import re
import subprocess
import sys
import wave
from pathlib import Path

HERE = Path(__file__).resolve().parent

# the sentence of each language: "The drilling is done. Now comes the routing." (Portuguese of Portugal for pt, Brazilian for pt-br)
STAGE = {
    "en": "The drilling is done. Now comes the routing.",
    "es": "El taladrado ha terminado. Ahora viene el fresado.",
    "it": "La foratura è terminata. Ora viene la fresatura.",
    "de": "Das Bohren ist fertig. Jetzt folgt das Fräsen.",
    "fr": "Le perçage est terminé. Vient maintenant le fraisage.",
    "pt": "A furação está concluída. Segue-se a fresagem.",
    "zh": "钻孔已完成。接下来是铣削。",
    "ja": "穴あけが完了しました。次はルーター加工です。",
    "pt-br": "A furação terminou. Agora vem a fresagem.",
    "bg": "Пробиването приключи. Сега следва фрезоването.",
    "cs": "Vrtání je hotové. Nyní následuje frézování.",
    "da": "Boringen er færdig. Nu kommer fræsningen.",
    "nl": "Het boren is klaar. Nu volgt het frezen.",
    "et": "Puurimine on valmis. Nüüd tuleb freesimine.",
    "fi": "Poraus on valmis. Nyt on jyrsinnän vuoro.",
    "el": "Η διάτρηση ολοκληρώθηκε. Τώρα ακολουθεί το φρεζάρισμα.",
    "hu": "A fúrás kész. Most jön a marás.",
    "lv": "Urbšana ir pabeigta. Tagad seko frēzēšana.",
    "no": "Boringen er ferdig. Nå kommer fresingen.",
    "pl": "Wiercenie zakończone. Teraz frezowanie.",
    "ro": "Găurirea s-a terminat. Urmează frezarea.",
    "sk": "Vŕtanie je hotové. Teraz nasleduje frézovanie.",
    "sl": "Vrtanje je končano. Zdaj sledi rezkanje.",
    "sv": "Borrningen är klar. Nu kommer fräsningen.",
    "uk": "Свердління завершено. Тепер буде фрезерування.",
    "ru": "Сверление завершено. Теперь следует фрезерование.",
    "sq": "Shpimi mbaroi. Tani vjen frezimi.",
    "eu": "Zulaketa amaitu da. Orain fresaketa dator.",
}

# the speed of the voices: Piper's length scale (above 1 is slower). The notices of this folder are spoken a little slower than usual;
# check that the new file lasts as the sentence should against finished.wav and change it with --length-scale if it does not.
LENGTH_SCALE = 1.15

LINE = re.compile(r"^([A-Za-z-]+)/([fm]): (\S+) - (.*)$")
SPEAKER = re.compile(r"speaker (\d+)")


def voices():
    """The voices of SOURCES.txt as a list of (language, gender, voice name, speaker or None)."""
    found = []
    for raw in (HERE / "SOURCES.txt").read_text(encoding="utf-8").splitlines():
        m = LINE.match(raw.strip())
        if not m:
            continue
        lang, gender, name, rest = m.groups()
        sp = SPEAKER.search(rest)
        found.append((lang, gender, name, int(sp.group(1)) if sp else None))
    return found


def make(model, speaker, text, out, length_scale, python):
    cmd = [python, "-m", "piper", "-m", str(model), "-f", str(out), "--length-scale", str(length_scale)]
    if speaker is not None:
        cmd += ["-s", str(speaker)]
    cmd += ["--", text]
    done = subprocess.run(cmd, capture_output=True, text=True)
    if done.returncode != 0:
        return (done.stderr or done.stdout).strip().splitlines()[-1:] or ["Piper failed"]
    # the same format as the other notices: 16 bits, mono (any rate)
    with wave.open(str(out), "rb") as w:
        if w.getnchannels() != 1 or w.getsampwidth() != 2:
            out.unlink()
            return [f"the file is not 16 bits mono ({w.getnchannels()} channels, {8 * w.getsampwidth()} bits)"]
    return None


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--models", type=Path, help="the folder with the .onnx models of the voices")
    ap.add_argument("--only", help="one voice, LANGUAGE/GENDER (for example es/f)")
    ap.add_argument("--force", action="store_true", help="make the file again where it exists")
    ap.add_argument("--list", action="store_true", help="only show what would be done")
    ap.add_argument("--length-scale", type=float, default=LENGTH_SCALE)
    ap.add_argument("--python", default=sys.executable, help="the Python that has Piper")
    args = ap.parse_args()
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    todo = [v for v in voices() if not args.only or f"{v[0]}/{v[1]}" == args.only]
    if not todo:
        print("no voice to do (see --only and SOURCES.txt)")
        return 1
    missing, failed, made = [], [], 0
    # one voice at a time: nothing runs in parallel
    for lang, gender, name, speaker in todo:
        out = HERE / lang / gender / "stage.wav"
        label = f"{lang}/{gender} {name}"
        text = STAGE.get(lang)
        if text is None:
            missing.append(f"{label}: no sentence in STAGE")
            continue
        if out.exists() and not args.force:
            print(f"{label}: already there")
            continue
        if args.list or args.models is None:
            print(f"{label}: would say \"{text}\"")
            continue
        model = args.models / f"{name}.onnx"
        if not model.is_file():
            missing.append(f"{label}: model not found ({model})")
            continue
        why = make(model, speaker, text, out, args.length_scale, args.python)
        if why:
            failed.append(f"{label}: {why[0]}")
        else:
            made += 1
            print(f"{label}: made")
    for line in missing:
        print("MISSING", line)
    for line in failed:
        print("FAILED", line)
    if not args.list and args.models is None:
        print("give --models to make the files")
    print(f"{made} made, {len(missing)} missing, {len(failed)} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
