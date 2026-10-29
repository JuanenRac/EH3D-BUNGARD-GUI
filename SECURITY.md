# Security policy

EH3D-BUNGARD-GUI opens files that other people may send: drilling files, HP-GL plot files and ZIP archives with a job inside. The readers are written not to panic on malformed input; a command they do not understand is counted and ignored, a name in an archive that would leave the work folder is not listed, and no file is read past a size limit.

What this program does **not** do: it is not a safety system and it does not handle limit switches or an emergency stop. It drives a machine through the engines of EH3D-DRILLPRO and EH3D-ROUTEPRO and has only run on the simulator. Do not connect a machine to it without reading the notes of those two programs.

## Reporting a problem

Please report a vulnerability privately to **electrohobby3d@gmail.com**, with the file or the steps that show it. Do not open a public issue for it. You will get an answer as soon as possible; this is a one-person project.
