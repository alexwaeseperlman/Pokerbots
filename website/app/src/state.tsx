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
import { UserProfile } from "@bindings/UserProfile";

export const apiUrl = window.location.origin + "/api";
export const authUrl = window.location.origin + "/auth";

const googleSigninUrlAtom = atom<Promise<string>>(
  fetch(`${authUrl}/oauth/google/client-id`)
    .then((res) => res.json())
    .then(
      (clientId) =>
        `https://accounts.google.com/o/oauth2/auth?` +
        `scope=${encodeURIComponent(
          `https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile`
        )}&` +
        `client_id=${encodeURIComponent(clientId)}&` +
        `redirect_uri=${encodeURIComponent(
          `${window.location.origin}/login/google`
        )}&` +
        `response_type=code&` +
        `prompt=select_account`
    )
);

export const useGoogleSigninUrl = () => useAtomValue(googleSigninUrlAtom);

const microsoftSigninUrlAtom = atom<Promise<string>>(
  fetch(`${authUrl}/oauth/microsoft/client-id`)
    .then((res) => res.json())
    .then(
      (clientId) =>
        `https://login.microsoftonline.com/common/oauth2/v2.0/authorize?` +
        `client_id=${encodeURIComponent(clientId)}&` +
        `redirect_uri=${encodeURIComponent(
          `${window.location.origin}/login/microsoft`
        )}&` +
        `response_type=code&` +
        `response_mode=query&` +
        `scope=User.Read&` +
        `prompt=select_account`
    )
);

export const useMicrosoftSigninUrl = () => useAtomValue(microsoftSigninUrlAtom);

// choose default value based on route
const teamAtom = atomFamily<
  string | null,
  PrimitiveAtom<Promise<TeamWithMembers<User> | null>>
>((param) => atom(fetchTeam(param)));

const userAtom = atom<Promise<User | null>>(
  fetch(`${apiUrl}/my-account`)
    .then((res) => res.json())
    .catch(() => null)
);

const profileAtom = atom<Promise<UserProfile | null>>(
  fetch(`${apiUrl}/profile`)
    .then((res) => res.json())
    .catch(() => null)
);

export const useAuth = (selectedTeam: string | null) => {
  const [user, setUser] = useAtom(userAtom);
  const [team, setTeam] = useAtom(teamAtom(selectedTeam ?? null));
  const [profile, setProfile] = useAtom(profileAtom);

  const update = () => {
    setTeam(fetchTeam(selectedTeam));

    setUser(
      fetch(`${apiUrl}/my-account`)
        .then((res) => res.json())
        .catch(() => null)
    );

    setProfile(
      fetch(`${apiUrl}/profile`)
        .then((res) => res.json())
        .catch(() => null)
    );
  };
  return [user, team, profile, update] as const;
};

export const useUser = () => {
  const [user, _team, _profile, update] = useAuth(null);
  return [user, update] as const;
};

export const useTeam = (selectedTeam: string | null) => {
  const [_user, team, _profile, update] = useAuth(selectedTeam);
  return [team, update] as const;
};

export const useProfile = () => {
  const [user, team, profile, update] = useAuth(null);
  return [profile, update] as const;
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
            const out: TeamWithMembers<User> = {
              ...teams.TeamsWithMembers[0],
            };
            return out;
          }
          return null;
        })
        .catch(() => null)
    : fetch(`${apiUrl}/my-team`)
        .then((res) => res.json())
        .then((team) => team as TeamWithMembers<User>)
        .catch(() => null);
}
