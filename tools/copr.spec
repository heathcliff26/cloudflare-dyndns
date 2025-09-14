%global debug_package %{nil}

Name:           cloudflare-dyndns
Version:        0
Release:        %autorelease
Summary:        DynDNS client or relay server for cloudflare
%global package_id io.github.heathcliff26.%{name}

License:        Apache-2.0
URL:            https://github.com/heathcliff26/%{name}
Source:         %{url}/archive/refs/tags/v%{version}.tar.gz

BuildRequires: golang >= 1.24

%global _description %{expand:
DynDNS client or relay server for cloudflare, implemented in golang.}

%description %{_description}

%prep
%autosetup -n %{name}-%{version} -p1

%build
export RELEASE_VERSION="%{version}-%{release}"
make build

%install
install -D -m 0755 bin/%{name} %{buildroot}%{_bindir}/%{name}
install -D -m 0644 tools/%{name}-client.service %{buildroot}%{_prefix}/lib/systemd/system/%{name}-client.service
install -D -m 0644 tools/%{name}-relay.service %{buildroot}%{_prefix}/lib/systemd/system/%{name}-relay.service
install -D -m 0644 tools/%{name}-server.service %{buildroot}%{_prefix}/lib/systemd/system/%{name}-server.service
install -D -m 0644 examples/example-config.yaml %{buildroot}%{_sysconfdir}/%{name}/config.yaml
install -D -m 0644 %{package_id}.metainfo.xml %{buildroot}/%{_datadir}/metainfo/%{package_id}.metainfo.xml

%post
systemctl daemon-reload

%preun
for mode in "client" "relay" "server"; do
  if [ $1 == 0 ]; then #uninstall
    systemctl unmask %{name}-${mode}.service
    systemctl stop %{name}-${mode}.service
    systemctl disable %{name}-${mode}.service
    echo "Clean up %{name}-${mode} service"
  fi
done

%postun
if [ $1 == 0 ]; then #uninstall
  systemctl daemon-reload
  systemctl reset-failed
fi

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_prefix}/lib/systemd/system/%{name}-client.service
%{_prefix}/lib/systemd/system/%{name}-relay.service
%{_prefix}/lib/systemd/system/%{name}-server.service
%{_sysconfdir}/%{name}/config.yaml
%dir %{_sysconfdir}/%{name}
%{_datadir}/metainfo/%{package_id}.metainfo.xml

%changelog
%autochangelog
