import {
  Atom,
  atom,
  PrimitiveAtom,
  useAtom,
  useAtomValue,
  useSetAtom,
  WritableAtom,
} from "jotai";
import { atomFamily, atomWithStorage } from "jotai/utils";
import { useEffect } from "react";
import { matchPath } from "react-router-dom";
import { UserData } from "@bindings/UserData";
import { Game } from "@bindings/Game";
import { Bot } from "@bindings/Bot";
import { TeamWithMembers } from "@bindings/TeamWithMembers";
import { TeamData } from "@bindings/TeamData";
import { TeamsResponse } from "@bindings/TeamsResponse";

export const apiUrl = window.location.origin + "/api";

const userAtom = atom<Promise<UserData | null>>(
  fetch(`${apiUrl}/my-account`)
    .then((res) => res.json())
    .catch(() => null)
);

export const useUser = () => {
  const [user, setUser] = useAtom(userAtom);
  const fetchUser = async () => {
    setUser(
      fetch(`${apiUrl}/my-account`)
        .then((res) => res.json())
        .catch(() => null)
    );
  };
  return [user, fetchUser] as const;
};

function fetchTeam(team: string | null) {
  return team
    ? fetch(`${apiUrl}/teams?ids=${team ?? ""}&fill_members=true`)
        .then((res) => res.json())
        .then((teams: TeamsResponse) => {
          if ("TeamsWithMembers" in teams) {
            const out: TeamData = { ...teams.TeamsWithMembers[0], invites: [] };
            return out;
          }
          return null;
        })
        .catch(() => null)
    : fetch(`${apiUrl}/my-team`)
        .then((res) => res.json())
        .then((team) => team as TeamData)
        .catch(() => null);
}

// choose default value based on route
const teamAtom = atomFamily<
  string | null,
  PrimitiveAtom<Promise<TeamData | null>>
>((param) => atom(fetchTeam(param)));

export const useTeam = (selectedTeam: string | null) => {
  const [team, setTeam] = useAtom(teamAtom(selectedTeam));
  const fetch = () => {
    setTeam(fetchTeam(selectedTeam));
  };
  return [team, fetch] as const;
};
