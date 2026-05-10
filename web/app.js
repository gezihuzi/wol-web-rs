const tokenInput = document.getElementById("token");
const devicesDiv = document.getElementById("devices");
const result = document.getElementById("result");

function getToken() {
    return tokenInput.value;
}

async function loadDevices() {
    try {
        const res = await fetch("/devices");
        const devices = await res.json();
        devicesDiv.innerHTML = "";
        if (devices.length === 0) {
            devicesDiv.innerHTML = "<p style='color:#999'>暂无设备，请在下方添加</p>";
            return;
        }
        devices.forEach((d) => {
            const row = document.createElement("div");
            row.className = "device-row";

            const nameSpan = document.createElement("span");
            nameSpan.className = "name";
            nameSpan.textContent = d.name;

            const macSpan = document.createElement("span");
            macSpan.className = "mac";
            macSpan.textContent = d.mac;

            const wakeBtn = document.createElement("button");
            wakeBtn.textContent = "唤醒";
            wakeBtn.onclick = () => wake(d.name);

            const delBtn = document.createElement("button");
            delBtn.className = "del";
            delBtn.textContent = "删除";
            delBtn.onclick = () => removeDevice(d.name);

            row.appendChild(nameSpan);
            row.appendChild(macSpan);
            row.appendChild(wakeBtn);
            row.appendChild(delBtn);
            devicesDiv.appendChild(row);
        });
    } catch (e) {
        result.textContent = "加载设备列表失败: " + e.message;
    }
}

async function wake(name) {
    const token = getToken();
    const params = new URLSearchParams({ device_name: name, token });
    try {
        const res = await fetch("/wake?" + params.toString());
        result.textContent = await res.text();
    } catch (e) {
        result.textContent = "请求失败: " + e.message;
    }
}

async function addDevice() {
    const name = document.getElementById("new_name").value.trim();
    const mac = document.getElementById("new_mac").value.trim();
    if (!name || !mac) {
        result.textContent = "名称和 MAC 地址不能为空";
        return;
    }
    const token = getToken();
    try {
        const res = await fetch("/devices", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name, mac, token }),
        });
        result.textContent = await res.text();
        document.getElementById("new_name").value = "";
        document.getElementById("new_mac").value = "";
        loadDevices();
    } catch (e) {
        result.textContent = "请求失败: " + e.message;
    }
}

async function removeDevice(name) {
    if (!confirm("确认删除设备 " + name + "?")) return;
    const token = getToken();
    try {
        const res = await fetch(
            "/devices?name=" + encodeURIComponent(name) + "&token=" + encodeURIComponent(token),
            { method: "DELETE" }
        );
        result.textContent = await res.text();
        loadDevices();
    } catch (e) {
        result.textContent = "请求失败: " + e.message;
    }
}

loadDevices();
