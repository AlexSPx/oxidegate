import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 1000 }, // Ramp-up
    { duration: '1m', target: 2000 },  // Sustained load
    { duration: '1m', target: 5000 },
    { duration: '1m', target: 10000 },
    { duration: '30s', target: 0 },    // Ramp-down
  ],
};

export default function () {
  const res = http.get('http://localhost:3000/hello_world');
  
  check(res, {
    'status 200': (r) => r.status === 200,
  });

  sleep(0.1);
}