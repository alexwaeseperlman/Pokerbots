import { atom, useAtom, useAtomValue, useSetAtom, WritableAtom } from "jotai";
import { useEffect } from "react";

export type User = {
  email: string;
  display_name: string;
};
const userAtom = atom<User | null>(null);

export const useUser = () => {
  const [user, setUser] = useAtom(userAtom);
  // fetch user
  useEffect(() => {
    (async () => {
      const data: User = (
        await fetch("/api/my-account")
      ).json() as unknown as User;
      setUser(data);
    })();
  }, []);
  return user;
};

export type Team = {
  id: number;
  team_name: string;
  members: User[];
  owner: string;
  elo: number | null;
};

const teamAtom = atom<Team | null>(null);

export const useTeam = () => {
  const [team, setTeam] = useAtom(teamAtom);
  // fetch team
  useEffect(() => {
    (async () => {
      const data: Team = (
        await fetch("/api/my-team")
      ).json() as unknown as Team;
      setTeam(data);
    })();
  }, []);
  return team;
};

export type ServerMessage = {
  message: string;
  type: "success" | "error";
};
const serverMessageAtom = atom<ServerMessage | null>(null);