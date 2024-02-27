import {
  Accordion,
  AccordionDetails,
  AccordionGroup,
  AccordionSummary,
  Card,
  CardContent,
  Link,
  Typography,
} from "@mui/joy";
import React from "react";

export default function FAQ() {
  return (
    <Card>
      <CardContent>
        <Typography level="h3" color="inherit">
          FAQ
        </Typography>
      </CardContent>
      <CardContent>
        <AccordionGroup>
          <Accordion>
            <AccordionSummary>What is UPAC?</AccordionSummary>
            <AccordionDetails>
              <Typography>
                UPAC stands for University Poker Algorithm Competition. It is a
                competition where students from different universities compete
                to create the best poker bot.
              </Typography>
            </AccordionDetails>
          </Accordion>
          <Accordion>
            <AccordionSummary>Can I compete on a team?</AccordionSummary>
            <AccordionDetails>
              <Typography>
                Yes! You can create a team and add your friends to it. You can
                then submit a bot together.
              </Typography>
            </AccordionDetails>
          </Accordion>
          <Accordion>
            <AccordionSummary>Who is elligible to compete?</AccordionSummary>
            <AccordionDetails>
              <Typography>
                Right now anyone is elligible, university student or not! The
                website is open so that anyone can create an account and submit
                a bot. Next year we plan to host a university only competition
                with prizes.
              </Typography>
            </AccordionDetails>
          </Accordion>
          <Accordion>
            <AccordionSummary>Who is behind UPAC?</AccordionSummary>
            <AccordionDetails>
              <Typography>
                UPAC was created by a group of students from McGill University.
                We started talking about strategies in our poker club and
                decided it would be fun to create a competition around it. After
                we started developing the website we realized that MIT hosts a
                similar event, but it's not open to the public. We decided to
                make our event open to everyone and we are excited to see the
                results.
              </Typography>
            </AccordionDetails>
          </Accordion>
          <Accordion>
            <AccordionSummary>
              Do I have to use a particular language/technology in particular?
            </AccordionSummary>
            <AccordionDetails>
              <Typography>
                Nope! You can use any language or technology you want. You
                submit a json file with a build command and a run command, and
                we will take care of the rest. Communication with the engine is
                done through stdin and stdout. For more information, check out
                the <Link href="https://docs.upac.dev/">documentation</Link>.
              </Typography>
            </AccordionDetails>
          </Accordion>
        </AccordionGroup>
      </CardContent>
    </Card>
  );
}
