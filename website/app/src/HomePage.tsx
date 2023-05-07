import React from "react";
import mainImage from "../static/assets/main.png";
import testImage from "../static/assets/test.webp";
import buildImage from "../static/assets/build.webp";
import competeImage from "../static/assets/compete.webp";

export default function HomePage() {
  return (
    <>
      <img
        src={mainImage}
        className="floating-picture"
        alt="fancy poker visual"
      />
      <h2>Build bots that win at Poker.</h2>
      <h5>
        PokerZero is a contest that challenges students from around the world to
        create the best poker bot using math, computer science, and economics.
      </h5>
      <div className="email-container">
        <input
          type="text"
          id="Email"
          className="email"
          placeholder="Your email"
        />
        <button className="join-button">Join</button>
      </div>
      <h6>Sign up for updates on PokerZero. No spam.</h6>
      <h3>How to play</h3>
      <section className="picture-section">
        <div className="picture-container">
          <div className="picture-wrapper">
            <img src={buildImage} alt="Picture 1" />
            <div className="picture-caption">
              <h4>Build.</h4>
              <p>
                With your team, design and code your bot in the programming
                language of your choice
              </p>
            </div>
          </div>

          <div className="picture-wrapper">
            <img src={testImage} alt="Picture 2" />
            <div className="picture-caption">
              <h4>Test.</h4>
              <p>
                Keep evaluating your bot on our Poker Engine until you are
                satisfied with its performance
              </p>
            </div>
          </div>

          <div className="picture-wrapper">
            <img src={competeImage} alt="Picture 3" />
            <div className="picture-caption">
              <h4>Compete.</h4>
              <p>
                Compete against other teams for a chance to win exciting prizes
                (and glory)!
              </p>
            </div>
          </div>
        </div>
      </section>
      <h3>That's it for now!</h3>
    </>
  );
}
