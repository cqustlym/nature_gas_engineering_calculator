document.getElementById("loginForm").addEventListener("submit", async (e) => {
    e.preventDefault();
    const username = document.getElementById("username").value;
    const password = document.getElementById("password").value;
    const errBox = document.getElementById("err");

    const res = await fetch("/api/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
    });

    if (res.ok) { // 200
        window.location.href = "/index.html";
    } else { // 401 或其他
        errBox.textContent = "用户名或密码错误";
    }
});