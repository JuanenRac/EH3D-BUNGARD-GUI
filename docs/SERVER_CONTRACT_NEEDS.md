# What the window needs from the EH3D server

The window sends its jobs to the server through the engines of EH3D-DRILLPRO and EH3D-ROUTEPRO (`POST /api/v1/machines/{machine}/jobs`). This page lists the
changes of that interface that the programs already send and that the server has to take; until it does, the programs fall back to the old behaviour and say so.

## Chained tool changes (`rack`)

A drilling job followed by a routing job on a machine with the automatic tool changer can be chained: the drill is not put back in its place, and the routing does one
change from the place of the last drill to the place of the first cutter. The programs plan this locally. For a job on the server the request now carries it.

### The request

`POST /api/v1/machines/{machine}/jobs`, for `"kind": "drill"` and for `"kind": "route"`, gets two optional fields. A request without chaining is exactly the one of before and has neither.

```json
{
  "kind": "route",
  "request_version": 2,
  "rack": { "first": 4, "carry_in": true, "carry_out": false }
}
```

| Field | Meaning |
|---|---|
| `request_version` | `2` when `rack` is present; absent in the requests of the first version |
| `rack.first` | how many places of the rack the tools of the jobs before this one take (0 for the first job of the load); the tools of this job wait from the next place |
| `rack.carry_in` | the chuck comes holding the last tool of the job before: the first change of this job takes it to its place and picks the first tool of this one |
| `rack.carry_out` | this job leaves its last tool in the chuck for the next one: it does not put it back |

### What the server has to do

1. Accept `request_version` and `rack` in both kinds of job, plan the changes of the tools with them (the same plan as the programs make locally) and keep the values in the job.
2. Answer the job with `"rack_applied": true` when it used them. A server that takes the fields but does not plan with them must not say it did.
3. A server that does not know the fields may ignore them (the answer then has no `rack_applied`) or refuse the job with 400 or 422.

### What the programs do meanwhile

- If the server answers 400 or 422 to a request with `rack`, the program sends the job again without the two fields.
- If the server answers 200 without `"rack_applied": true`, the job runs as the server planned it.
- In both cases every tool goes back to its own place after its work, and the run window shows a red line saying the server did not chain the tool changes.

## Places of the tools in the bar (`rack.slots`)

The tools of a job do not have to be in the bar in the order the job uses them: the person says in which place each one is (the window of the positions in the bar). The programs plan
this locally. For a job on the server the same `rack` field carries it.

### The request

```json
{
  "kind": "drill",
  "request_version": 2,
  "rack": {
    "first": 0,
    "carry_in": false,
    "carry_out": true,
    "slots": [ { "tool": 1, "place": 6 }, { "tool": 2, "place": 3 } ],
    "carry_slot": 5
  }
}
```

| Field | Meaning |
|---|---|
| `rack.slots` | the place of each tool of this job: the tool number and the place of the rack it is in (1 to 14; 0 is the test pin). Absent when the person gave no places: the tools wait from the place `first + 1` in the order of the job, as before |
| `rack.carry_slot` | with `carry_in`, the place of the tool the chuck comes holding (the last tool of the job before); absent: the place `first` |

### What the server has to do

1. Take every tool of this job from the place in `slots` (a tool that is not in the list is taken from the place the order of the job gives it), put the tool the chuck holds back in its own place, and
   put the last tool back in its place at the end unless `carry_out`.
2. Refuse the job with 400 or 422 when a place is not defined in the machine file or is out of reach, and say which.
3. Answer with `"rack_applied": true` only when it planned with the places.

### What the programs do meanwhile

- A server that refuses the field gets the job again without `rack`: it plans the tools in the order of the job, and not in the places of the window. The run window shows the red line
  of the chained tool changes; **the tools of that job must then be in the bar in the order of the job.**
- A server that answers 200 without `"rack_applied": true` has done the same.
