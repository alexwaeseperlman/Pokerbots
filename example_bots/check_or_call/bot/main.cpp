#include <iostream>
using namespace std;

int vals[4] = {2, 5, 6, 7};
int inp[5] = {0};
int main() {
  int round = -1, position = 0;
  while (true) {
    char type;
    cin >> type;
    // cerr << type << ' ';
    if (type == 'E') {
      round = -1;
    } else if (type == 'P') {
      cin >> position;
      // cerr << position;
    } else if (type == 'C') {
      round++;
      char a;
      for (int i = 0; i < vals[round]; i++) {
        cin >> a;
        // cerr << a;
        cin >> a;
        // cerr << a << ' ';
      }
    } else {
      for (int i = 0; i < 5; i++) {
        cin >> inp[i];
      }
      for (int i = 0; i < 5; i++) {
        // cerr << inp[i] << " ";
      }

      if (inp[position + 1] == inp[0])
        cout << 'X' << endl;
      else
        cout << 'C' << endl;
      cout.flush();
    }
    // cerr << endl;
  }
}