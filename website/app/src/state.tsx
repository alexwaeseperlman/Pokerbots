import { atom, useAtom, useAtomValue, useSetAtom, WritableAtom } from "jotai";
import { useEffect } from "react";

export type User = {
  email: string;
  display_name: string;
};
const userAtom = atom<User | null | undefined>(
  (JSON.parse(localStorage.getItem("user") || "null") ?? undefined) as
    | User
    | null
    | undefined
);

export const useUser = () => {
  const [user, setUser] = useAtom(userAtom);
  // fetch user
  useEffect(() => {
    (async () => {
      const data: User = (await (
        await fetch(`${apiUrl}/my-account`)
      ).json()) as unknown as User;
      console.log(data);
      localStorage.setItem("user", JSON.stringify(data));
      setUser(data);
    })();
  }, []);
  return user;
};

export type Team = {
  id: number;
  team_name: string;
  members: User[];
  invites: string[];
  owner: string;
  elo: number | null;
};

const teamAtom = atom<Team | null | undefined>(
  (JSON.parse(localStorage.getItem("team") || "null") ?? undefined) as
    | Team
    | null
    | undefined
);

export const useTeam = () => {
  const [team, setTeam] = useAtom(teamAtom);
  const fetchTeam = async () => {
    const data: Team = (await (
      await fetch(`${apiUrl}/my-team`)
    ).json()) as unknown as Team;
    setTeam(data);
    localStorage.setItem("team", JSON.stringify(data));
  };
  // fetch team
  useEffect(() => {
    fetchTeam();
  }, []);
  return [team, fetchTeam] as const;
};

const pfpEndpointAtom = atom<string | null | undefined>(
  (localStorage.getItem("pfpEndpoint") ?? null ?? undefined) as
    | string
    | null
    | undefined
);
export const usePfpEndpoint = () => {
  const [pfpEndpoint, setPfpEndpoint] = useAtom(pfpEndpointAtom);
  const fetchPfpEndpoint = async () => {
    const data: string = (await (
      await fetch("/api/pfp-url")
    ).text()) as unknown as string;
    setPfpEndpoint(data);
    localStorage.setItem("pfpEndpoint", data);
  };
  // fetch pfpEndpoint
  useEffect(() => {
    fetchPfpEndpoint();
  }, []);
  return pfpEndpoint;
};
export const apiUrl = window.location.origin + "/api";
