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

// choose default value based on route
const teamAtom = atomFamily<
  string | null,
  PrimitiveAtom<Promise<TeamData | null>>
>((param) => atom(fetchTeam(param)));

const userAtom = atom<Promise<UserData | null>>(
  fetch(`${apiUrl}/my-account`)
    .then((res) => res.json())
    .catch(() => null)
);

export const useProfile = (selectedTeam: string | null) => {
  const [user, setUser] = useAtom(userAtom);
  const [team, setTeam] = useAtom(teamAtom(selectedTeam ?? null));

  const update = () => {
    setTeam(fetchTeam(selectedTeam));

    setUser(
      fetch(`${apiUrl}/my-account`)
        .then((res) => res.json())
        .catch(() => null)
    );
  };
  return [user, team, update] as const;
};

export const useUser = () => {
  const [user, team, update] = useProfile(null);
  return [user, update] as const;
};

export const useTeam = (selectedTeam: string | null) => {
  const [user, team, update] = useProfile(selectedTeam);
  return [team, update] as const;
};

function fetchTeam(team: string | null) {
  return team
    ? fetch(`${apiUrl}/teams?ids=${team ?? ""}&fill_members=true`)
        .then((res) => res.json())
        .then((teams: TeamsResponse) => {
          if (
            "TeamsWithMembers" in teams &&
            teams.TeamsWithMembers.length > 0
          ) {
            const invites = teams.TeamsWithMembers[0].invites;
            const out: TeamData = {
              ...teams.TeamsWithMembers[0],
              invites: invites ? invites.map((val) => val.code) : [],
            };
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
