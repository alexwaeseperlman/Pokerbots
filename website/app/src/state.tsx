import { atom, useAtom, useAtomValue, useSetAtom, WritableAtom } from "jotai";
import { useEffect } from "react";

export type User = {
  email: string;
  display_name: string;
};
const userAtom = atom<User | null | undefined>(undefined);

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
  invites: string[];
  owner: string;
  elo: number | null;
};

const teamAtom = atom<Team | null | undefined>(undefined);

export const useTeam = () => {
  const [team, setTeam] = useAtom(teamAtom);
  const fetchTeam = async () => {
    const data: Team = (await fetch("/api/my-team")).json() as unknown as Team;
    setTeam(data);
  };
  // fetch team
  useEffect(() => {
    fetchTeam();
  }, []);
  return [team, fetchTeam] as const;
};

export type ServerMessage = {
  message: string;
  type: "success" | "error";
};
const serverMessageAtom = atom<ServerMessage | null | undefined>(undefined);
