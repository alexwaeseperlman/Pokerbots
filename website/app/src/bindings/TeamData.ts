// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { UserData } from "./UserData";

export interface TeamData { id: number, name: string, members: Array<UserData>, owner: string, score: number | null, invites: Array<string>, active_bot: number | null, }