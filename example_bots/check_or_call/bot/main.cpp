#include <iostream>
#include <vector>
#include <cassert>
using namespace std;

int main() {
  int round = -1;

  while (true) {
    cerr << "Starting round" << endl;

    int position = 0;
    int curStack = 0, oppStack = 0;
    int curPush = 0, oppPush = 0;

    vector<string> hole;
    vector<string> community;
    string type;
    cin >> type;
    assert(type == "START");

    string pos; cin >> pos;
    if (pos == "BB") position = 1;

    while (type != "END") {
      if (type == "STACK") {
        cin >> curPush >> curStack >> oppPush >> oppStack;
        // Time to act!
        // R<x> - raise to <x> chips
        // C - call
        // F - fold
        cout << "C" << endl;
      }

      else if (type == "PREFLOP") {
        // we won't use this, but we need to take the input
        string x, y; cin >> x >> y;
        hole.push_back(x);
        hole.push_back(y);
        cerr << "Hole: " << hole[0] << " " << hole[1] << endl;
      }

      else if (type == "FLOP") {
        string x, y, z; cin >> x >> y >> z;
        community.push_back(x);
        community.push_back(y);
        community.push_back(z);
        cerr << "Flop: " << community[0] << " " << community[1] << " " << community[2] << endl;
      }

      else if (type == "TURN") {
        string x; cin >> x;
        community.push_back(x);
        cerr << "Turn: " << x << endl;
      }

      else if (type == "RIVER") {
        string x; cin >> x;
        community.push_back(x);
        cerr << "River: " << x << endl;
      }

      cin >> type;
    }

    assert(type == "END");
    cin >> type;
    if (type == "FOLD") {
      string loser; cin >> loser;
      cerr << loser << " folded." << endl;
    }
    else {
      assert(type == "SHOWDOWN");
      string type; cin >> type;

      if (type == "TIE") {
        cerr << "Tie." << endl;
      }
      else {
        assert(type == "WINNER");
        string winner; cin >> winner;

        cerr << winner << " won." << endl;
        cin >> type;

        if (type == "HIDDEN") {
          cerr << "Opponent cards are hidden." << endl;
        }
        else {
          assert(type == "SHOWN");
          string x, y; cin >> x >> y;
          cerr << "Opponent cards: " << x << " " << y << endl;
        }
      }

    }
  }
}
