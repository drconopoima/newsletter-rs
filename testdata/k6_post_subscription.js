import http from 'k6/http';
export default function () {
  const params = {
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
  };
  let randomnumber = String(Math.floor(Math.random() * (10000000000 - 1)));
  let data = `email=${randomnumber}%40drconopoima.com&name=${randomnumber}`;
  http.post('http://localhost:8000/subscription', data,params);
}
