# Linux Commands Cheatsheet

Quick reference organized by task category.

## Files and Directories

```bash
ls -la                          # list with permissions and hidden files
ls -lah --sort=size             # sorted by size
stat file                       # detailed file metadata
file somefile                   # detect file type
find /path -name "*.log" -mtime +30 -delete   # delete logs older than 30 days
find /path -type f -size +100M  # find files larger than 100MB
tree -L 2                       # directory tree, 2 levels deep
rsync -avz src/ dest/           # sync directories (preserves permissions)
tar czf archive.tar.gz dir/     # create compressed archive
tar xzf archive.tar.gz          # extract archive
```

## Text Processing

```bash
grep -rn "pattern" /path/       # recursive search with line numbers
grep -rli "pattern" /path/      # list files containing pattern
wc -l file                      # count lines
sort file | uniq -c | sort -rn  # frequency count
cut -d: -f1 /etc/passwd         # extract first field
awk '{print $1, $NF}' file      # print first and last columns
sed -i 's/old/new/g' file       # in-place replacement
diff -u file1 file2             # unified diff
```

## Process Management

```bash
ps aux                          # all processes
ps aux --sort=-%cpu | head      # top CPU consumers
pgrep -af "name"                # find by name with full command
pkill -f "pattern"              # kill by pattern
kill -TERM <pid>                # graceful shutdown
kill -KILL <pid>                # force kill
nohup command &                 # run detached from terminal
timeout 30s command             # kill after timeout
nice -n 10 command              # run with lower priority
renice -n 5 -p <pid>            # change priority of running process
```

## Network

```bash
ss -tlnp                        # listening TCP ports with process
ss -tunap                       # all TCP/UDP connections
curl -sS -o /dev/null -w "%{http_code}" http://host   # check HTTP status
wget -q --spider http://host    # test URL reachability
dig example.com                 # DNS lookup
host example.com                # simple DNS lookup
ip addr show                    # network interfaces and IPs
ip route show                   # routing table
ping -c 4 host                  # connectivity test
traceroute host                 # path to host
nc -zv host port                # test TCP port connectivity
tcpdump -i eth0 port 80         # capture packets (requires root)
```

## Disk and Storage

```bash
df -h                           # filesystem usage
df -ih                          # inode usage
du -sh /path/*                  # directory sizes
du -sh /path/* | sort -rh       # sorted by size
lsblk -f                        # block devices with filesystems
blkid                           # partition UUIDs and types
mount | column -t               # current mounts
fdisk -l                        # partition table (requires root)
iostat -x 1 5                   # disk I/O statistics
```

## Users and Permissions

```bash
whoami                          # current user
id                              # uid, gid, groups
who                             # logged-in users
last -10                        # recent logins
chmod 755 file                  # set permissions
chown user:group file           # change ownership
chown -R user:group dir/        # recursive ownership
getfacl file                    # view ACLs
setfacl -m u:user:rwx file     # set ACL entry
passwd user                     # change password
```

## System Information

```bash
uname -a                        # kernel and OS info
hostnamectl                     # hostname and OS details
uptime                          # system uptime and load
free -h                         # memory usage
cat /proc/cpuinfo | grep "model name" | head -1   # CPU info
nproc                           # number of CPUs
lscpu                           # detailed CPU architecture
dmesg -T | tail -50             # recent kernel messages with timestamps
```

## Package Management (Debian/Ubuntu)

```bash
apt update                      # refresh package index
apt list --upgradable           # show available upgrades
apt install -y pkg              # install package
apt remove pkg                  # remove package
apt purge pkg                   # remove package and config
apt autoremove -y               # remove unused dependencies
dpkg -l | grep pkg              # check installed package
dpkg -L pkg                     # list files installed by package
apt-cache policy pkg            # show available versions
```

## Package Management (RHEL/Fedora)

```bash
dnf check-update                # check for updates
dnf install -y pkg              # install package
dnf remove pkg                  # remove package
dnf autoremove                  # remove unused dependencies
rpm -qa | grep pkg              # check installed package
rpm -ql pkg                     # list files installed by package
dnf info pkg                    # package details
```
