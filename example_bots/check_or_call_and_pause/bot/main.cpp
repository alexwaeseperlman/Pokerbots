#include <chrono>
#include <iostream>
#include <stdlib.h>
#include <thread>
#include <unistd.h>
using namespace std;

int vals[4] = {2, 5, 6, 7};
int inp[5] = {0};
int main() {
  int round = -1, position = 0;
  while (true) {
    char type;
    cin >> type;
    if (type == 'E') {
      round = -1;
      cerr << "Round ended." << endl << endl;
    } else if (type == 'P') {
      cin >> position;
      cerr << "My position is " << position << endl;
    } else if (type == 'C') {
      round++;
      char a;
      cerr << "Cards: ";
      for (int i = 0; i < vals[round]; i++) {
        cin >> a;
        cerr << a;
        cin >> a;
        cerr << a << ' ';
      }
      cerr << endl;
    } else {
      cerr << "Stacks: ";
      for (int i = 0; i < 5; i++) {
        cin >> inp[i];
      }
      for (int i = 0; i < 5; i++) {
        cerr << inp[i] << " ";
      }
      cerr << endl;
      std::this_thread::sleep_for(std::chrono::milliseconds(50));
      cerr << "Deciding on an action: ";
      if (inp[position + 1] == inp[0]) {
        cerr << "Check." << endl;
        cout << 'X' << endl;
      } else {
        cerr << "Call." << endl;
        cout << 'C' << endl;
      }
      cout.flush();
    }
    // cerr << endl;
  }
}
