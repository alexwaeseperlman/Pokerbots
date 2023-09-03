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
import { Game } from "@bindings/Game";
import { Bot } from "@bindings/Bot";
import { TeamWithMembers } from "@bindings/TeamWithMembers";
import { TeamsResponse } from "@bindings/TeamsResponse";
import { User } from "@bindings/User";

export const apiUrl = window.location.origin + "/api";
export const authUrl = window.location.origin + "/auth";

export const googleSigninUrl =
  `https://accounts.google.com/o/oauth2/auth?` +
  `scope=${encodeURIComponent(
    `https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile`
  )}&` +
  `client_id=${encodeURIComponent(import.meta.env.APP_GOOGLE_CLIENT_ID)}&` +
  `redirect_uri=${encodeURIComponent(
    `${window.location.origin}/login/google`
  )}&` +
  `response_type=code&` +
  `prompt=select_account`;

export const microsoftSigninUrl =
  `https://login.microsoftonline.com/common/oauth2/v2.0/authorize?` +
  `client_id=${encodeURIComponent(import.meta.env.APP_MICROSOFT_CLIENT_ID)}&` +
  `redirect_uri=${encodeURIComponent(
    `${window.location.origin}/login/microsoft`
  )}&` +
  `response_type=code&` +
  `response_mode=query&` +
  `scope=User.Read&` +
  `prompt=select_account`;

// choose default value based on route
const teamAtom = atomFamily<
  string | null,
  PrimitiveAtom<Promise<TeamWithMembers | null>>
>((param) => atom(fetchTeam(param)));

const userAtom = atom<Promise<User | null>>(
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
            const out: TeamWithMembers = {
              ...teams.TeamsWithMembers[0],
            };
            return out;
          }
          return null;
        })
        .catch(() => null)
    : fetch(`${apiUrl}/my-team`)
        .then((res) => res.json())
        .then((team) => team as TeamWithMembers)
        .catch(() => null);
}
