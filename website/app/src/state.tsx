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

// choose default value based on route
const teamAtom = atomFamily<
  string | null,
  PrimitiveAtom<Promise<TeamData | null>>
>((param) =>
  atom(
    param
      ? fetch(`${apiUrl}/teams?ids=${param ?? ""}&fill_members=true`)
          .then((res) => res.json())
          .then((teams) => teams[0])
          .catch(() => null)
      : fetch(`${apiUrl}/my-team`)
          .then((res) => res.json())
          .catch(() => null)
  )
);

export const useTeam = (selectedTeam: string | null) => {
  const [team, setTeam] = useAtom(teamAtom(selectedTeam));
  const fetchTeam = () => {
    if (!selectedTeam)
      setTeam(
        fetch(`${apiUrl}/my-team`)
          .then((res) => res.json())
          .catch(() => null)
      );
    else {
      setTeam(
        fetch(`${apiUrl}/teams?ids=${selectedTeam}&fill_members=true`)
          .then((res) => res.json())
          .then((teams) => teams[0])
          .catch(() => null)
      );
    }
  };
  return [team, fetchTeam] as const;
};
