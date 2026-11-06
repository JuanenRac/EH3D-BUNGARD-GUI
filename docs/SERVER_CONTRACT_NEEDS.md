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
